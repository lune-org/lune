use std::{
    env::consts::EXE_EXTENSION,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{bail, Context, Error, Result};
use async_compression::tokio::bufread::DeflateDecoder;
use clap::Parser;
use console::style;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt as _},
};

use crate::standalone::metadata::Metadata;

const TARGET_BASE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    BaseDirs::new()
        .unwrap()
        .home_dir()
        .to_path_buf()
        .join(".lune")
        .join("target")
        .join(env!("CARGO_PKG_VERSION"))
});

// Build a standalone executable
#[derive(Debug, Clone, Parser)]
pub struct BuildCommand {
    /// The path to the input file
    pub input: PathBuf,

    /// The path to the output file - defaults to the
    /// input file path with an executable extension
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    #[clap(short, long)]
    pub target: Option<String>,

    #[clap(short, long)]
    pub base: Option<PathBuf>,
}

// TODO: Currently, the file we are patching is user provided, so we should
// probably check whether the binary is a valid lune base binary first

impl BuildCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let mut output_path = self
            .output
            .unwrap_or_else(|| self.input.with_extension(EXE_EXTENSION));

        let input_path_displayed = self.input.display();

        // Try to read the input file
        let source_code = fs::read(&self.input)
            .await
            .context("failed to read input file")?;

        // Dynamically derive the base executable path based on the CLI arguments provided
        let base_exe_path = match get_base_exe_path(self.base, self.target, &mut output_path).await
        {
            Ok(path) => Some(path),
            Err(err) => {
                let inner_err = err.downcast::<BasePathDiscoveryError<()>>();

                if let Err(other_err) = inner_err {
                    bail!(
                        "Encountered an error while handling cross-compilation flags: {}",
                        other_err
                    );
                }

                // If there the downcasted error was ok, it is safe to continue since
                // neither the --base nor the --target flags were set
                None
            }
        };

        // Read the contents of the lune interpreter as our starting point
        println!(
            "{} standalone binary using {}",
            style("Compile").green().bold(),
            style(input_path_displayed).underlined()
        );
        let patched_bin = Metadata::create_env_patched_bin(base_exe_path, source_code.clone())
            .await
            .context("failed to create patched binary")?;

        // And finally write the patched binary to the output file
        println!(
            "   {} standalone binary to {}",
            style("Write").blue().bold(),
            style(output_path.display()).underlined()
        );
        write_executable_file_to(output_path, patched_bin).await?;

        Ok(ExitCode::SUCCESS)
    }
}

async fn write_executable_file_to(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        options.mode(0o755); // Read & execute for all, write for owner
    }

    let mut file = options.open(path).await?;
    file.write_all(bytes.as_ref()).await?;

    Ok(())
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum BasePathDiscoveryError<T> {
    #[error("decompression error")]
    Decompression(T),
    #[error("precompiled base for target not found for {target}")]
    TargetNotFound { target: String },
    #[error("failed to download precompiled binary base")]
    DownloadError(T),
    #[error("a generic error related to an io operation occurred")]
    IoError(T),
    #[error("neither a custom base path or precompiled target name provided")]
    None,
}

async fn get_base_exe_path(
    base: Option<PathBuf>,
    target: Option<String>,
    output_path: &mut PathBuf,
) -> Result<PathBuf> {
    if let Some(base) = base {
        output_path.set_extension(
            base.extension()
                .expect("failed to get extension of base binary"),
        );

        Ok(base)
    } else if let Some(target_inner) = target {
        let target_exe_extension = match target_inner.as_str() {
            "windows-x86_64" => "exe",
            _ => "bin",
        };

        let path = TARGET_BASE_DIR.join(format!("lune-{target_inner}.{target_exe_extension}"));

        output_path.set_extension(if target_exe_extension == "bin" {
            ""
        } else {
            target_exe_extension
        });

        if !TARGET_BASE_DIR.exists() {
            fs::create_dir_all(TARGET_BASE_DIR.to_path_buf())
                .await
                .map_err(BasePathDiscoveryError::IoError)?;
        }

        if !path.exists() {
            println!("Requested target hasn't been downloaded yet, attempting to download");

            let release_url = format!(
                "https://github.com/lune-org/lune/releases/download/v{ver}/lune-{ver}-{target}.zip",
                ver = env!("CARGO_PKG_VERSION"),
                target = target_inner
            );

            let target_full_display = release_url
                .split('/')
                .last()
                .unwrap_or("lune-UNKNOWN-UNKNOWN")
                .replace("zip", target_exe_extension);

            println!(
                "{} target {}",
                style("Download").green().bold(),
                target_full_display
            );

            // Maybe we should use the custom net client used in `@lune/net`
            let resp = reqwest::get(release_url).await.map_err(|err| {
                eprintln!(
                    "   {} Unable to download base binary found for target `{}`",
                    style("Download").red().bold(),
                    target_inner,
                );

                BasePathDiscoveryError::DownloadError::<Error>(err.into())
            })?;

            let resp_status = resp.status();

            if resp_status != 200 && !resp_status.is_redirection() {
                eprintln!(
                    "   {} No precompiled base binary found for target `{}`",
                    style("Download").red().bold(),
                    target_inner
                );

                println!("{}: {}", style("HINT").yellow(), style("Perhaps try providing a path to self-compiled target with the `--base` flag").italic());

                return Err(BasePathDiscoveryError::TargetNotFound::<String> {
                    target: target_inner,
                }
                .into());
            }

            let compressed_reader = resp
                .bytes()
                .await
                .map_err(BasePathDiscoveryError::IoError)?;
            let mut decompressed_bytes = vec![];

            // This errors, so idk what decoder to use
            DeflateDecoder::new(compressed_reader.as_ref())
                .read_to_end(&mut decompressed_bytes)
                .await
                .map_err(BasePathDiscoveryError::Decompression)?;

            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await
                .map_err(BasePathDiscoveryError::IoError)?
                .write_all(&decompressed_bytes)
                .await
                .map_err(BasePathDiscoveryError::IoError)?;

            println!(
                "  {} {}",
                style("Downloaded").blue(),
                style(target_full_display).underlined()
            );
        }

        Ok(path)
    } else {
        Err(BasePathDiscoveryError::<()>::None.into())
    }
}
