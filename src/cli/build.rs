use std::{
    env::consts::EXE_EXTENSION,
    io::Cursor,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use async_zip::base::read::seek::ZipFileReader;
use clap::Parser;
use console::style;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

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

    /// The target to compile for - defaults to the host triple
    #[clap(short, long)]
    pub target: Option<String>,

    /// The path to the base executable to use - defaults to
    /// the currently running executable, used for cross-compilation
    /// for targets not directly supported
    #[clap(short, long)]
    pub base: Option<PathBuf>,
}

impl BuildCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let output_path = self
            .output
            .unwrap_or_else(|| self.input.with_extension(EXE_EXTENSION));

        let input_path_displayed = self.input.display();

        // Try to read the input file
        let source_code = fs::read(&self.input)
            .await
            .context("failed to read input file")?;

        // Dynamically derive the base executable path based on the CLI arguments provided
        let (to_cross_compile, base_exe_path, output_path) =
            get_base_exe_path(self.base, self.target, output_path).await?;

        // Read the contents of the lune interpreter as our starting point
        println!(
            "{} standalone binary using {}",
            style("Compile").green().bold(),
            style(input_path_displayed).underlined()
        );
        let patched_bin = Metadata::create_env_patched_bin(
            if to_cross_compile {
                Some(base_exe_path)
            } else {
                None
            },
            source_code.clone(),
        )
        .await
        .context("failed to create patched binary")?;

        // And finally write the patched binary to the output file
        println!(
            "   {} standalone binary to {}",
            style("Write").blue().bold(),
            style(output_path.display()).underlined()
        );
        write_file_to(output_path, patched_bin, 0o755).await?; // Read & execute for all, write for owner

        Ok(ExitCode::SUCCESS)
    }
}

/// Wrapper function to asynchronously create a file at the given path with the given contents and permissions
async fn write_file_to(
    path: impl AsRef<Path>,
    bytes: impl AsRef<[u8]>,
    perms: u32,
) -> Result<File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).read(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        options.mode(perms);
    }

    let mut file = options.open(path).await?;
    file.write_all(bytes.as_ref()).await?;

    Ok(file)
}

/// Possible ways in which the discovery and/or download of a base binary's path can error
#[derive(Debug, Error)]
pub enum BasePathDiscoveryError {
    /// An error in the decompression of the precompiled target
    #[error("decompression error")]
    Decompression(#[from] async_zip::error::ZipError),
    #[error("precompiled base for target not found for {target}")]
    TargetNotFound { target: String },
    /// An error in the precompiled target download process
    #[error("failed to download precompiled binary base, reason: {0}")]
    DownloadError(#[from] reqwest::Error),
    /// An IO related error
    #[error("a generic error related to an io operation occurred, details: {0}")]
    IoError(#[from] anyhow::Error),
}

/// Discovers the path to the base executable to use for cross-compilation
async fn get_base_exe_path(
    base: Option<PathBuf>,
    target: Option<String>,
    output_path: PathBuf,
) -> Result<(bool, PathBuf, PathBuf), BasePathDiscoveryError> {
    if let Some(base) = base {
        Ok((
            true,
            base.clone(),
            output_path.with_extension(
                base.extension()
                    .expect("failed to get extension of base binary"),
            ),
        ))
    } else if let Some(target_inner) = target {
        let target_exe_extension = match target_inner.as_str() {
            "windows-x86_64" => "exe",
            _ => "bin",
        };

        let path = TARGET_BASE_DIR.join(format!("lune-{target_inner}.{target_exe_extension}"));

        // Create the target base directory in the lune home if it doesn't already exist
        if !TARGET_BASE_DIR.exists() {
            fs::create_dir_all(TARGET_BASE_DIR.to_path_buf())
                .await
                .map_err(anyhow::Error::from)
                .map_err(BasePathDiscoveryError::IoError)?;
        }

        // If a cached target base executable doesn't exist, attempt to download it
        if !path.exists() {
            println!("Requested target hasn't been downloaded yet, attempting to download");
            cache_target(target_inner, target_exe_extension, &path).await?;
        }

        Ok((
            true,
            path,
            output_path.with_extension(if target_exe_extension == "bin" {
                ""
            } else {
                target_exe_extension
            }),
        ))
    } else {
        Ok((false, PathBuf::new(), PathBuf::new()))
    }
}

async fn cache_target(
    target: String,
    target_exe_extension: &str,
    path: &PathBuf,
) -> Result<(), BasePathDiscoveryError> {
    let release_url = format!(
        "https://github.com/lune-org/lune/releases/download/v{ver}/lune-{ver}-{target}.zip",
        ver = env!("CARGO_PKG_VERSION"),
        target = target
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

    let resp = reqwest::get(release_url).await.map_err(|err| {
        eprintln!(
            "   {} Unable to download base binary found for target `{}`",
            style("Download").red().bold(),
            target,
        );

        BasePathDiscoveryError::DownloadError(err)
    })?;

    let resp_status = resp.status();

    if resp_status != 200 && !resp_status.is_redirection() {
        eprintln!(
            "   {} No precompiled base binary found for target `{}`",
            style("Download").red().bold(),
            target
        );

        println!(
            "{}: {}",
            style("HINT").yellow(),
            style("Perhaps try providing a path to self-compiled target with the `--base` flag")
                .italic()
        );

        return Err(BasePathDiscoveryError::TargetNotFound { target });
    }

    // Wrap the request response in bytes so that we can decompress it, since `async_zip`
    // requires the underlying reader to implement `AsyncRead` and `Seek`, which `Bytes`
    // doesn't implement
    let compressed_data = Cursor::new(
        resp.bytes()
            .await
            .map_err(anyhow::Error::from)
            .map_err(BasePathDiscoveryError::IoError)?
            .to_vec(),
    );

    // Construct a decoder and decompress the ZIP file using deflate
    let mut decoder = ZipFileReader::new(compressed_data.compat())
        .await
        .map_err(BasePathDiscoveryError::Decompression)?;

    let mut decompressed = vec![];

    decoder
        .reader_without_entry(0)
        .await
        .map_err(BasePathDiscoveryError::Decompression)?
        .compat()
        .read_to_end(&mut decompressed)
        .await
        .map_err(anyhow::Error::from)
        .map_err(BasePathDiscoveryError::IoError)?;

    // Finally write the decompressed data to the target base directory
    write_file_to(&path, decompressed, 0o644)
        .await
        .map_err(BasePathDiscoveryError::IoError)?;

    println!(
        "  {} {}",
        style("Downloaded").blue(),
        style(target_full_display).underlined()
    );

    Ok(())
}
