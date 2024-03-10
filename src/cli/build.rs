use std::{
    env::consts::EXE_EXTENSION,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use tokio::{fs, io::AsyncWriteExt as _};

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
        let base_exe_path = get_base_exe_path(self.base, self.target, &mut output_path).await;

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

async fn get_base_exe_path(
    base: Option<PathBuf>,
    target: Option<String>,
    output_path: &mut PathBuf,
) -> Option<PathBuf> {
    if base.is_some() {
        output_path.set_extension(
            base.clone()
                .unwrap()
                .extension()
                .expect("failed to get extension of base binary"),
        );

        base
    } else if let Some(target_inner) = target {
        let target_exe_extension = match target_inner.as_str() {
            "windows-x86_64" => "exe",
            _ => "bin",
        };

        let path = TARGET_BASE_DIR.join(format!("lune-{}.{}", target_inner, target_exe_extension));

        output_path.set_extension(if target_exe_extension == "bin" {
            ""
        } else {
            target_exe_extension
        });

        if !TARGET_BASE_DIR.exists() {
            fs::create_dir_all(TARGET_BASE_DIR.to_path_buf())
                .await
                .ok()?;
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
            let dl_req = match reqwest::get(release_url).await {
                Err(_) => {
                    eprintln!(
                        "   {} Unable to download base binary found for target `{}`",
                        style("Download").red().bold(),
                        target_inner,
                    );

                    return None;
                }
                Ok(resp) => {
                    let resp_status = resp.status();

                    if resp_status != 200 && !resp_status.is_redirection() {
                        eprintln!(
                            "   {} No precompiled base binary found for target `{}`",
                            style("Download").red().bold(),
                            target_inner
                        );

                        println!("{}: {}", style("HINT").yellow(), style("Perhaps try providing a path to self-compiled target with the `--base` flag").italic());

                        return None;
                    }

                    resp
                }
            };

            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await
                .ok()?
                .write_all(&dl_req.bytes().await.ok()?)
                .await
                .ok()?;

            println!(
                "  {} {}",
                style("Downloaded").blue(),
                style(target_full_display).underlined()
            );
        }

        Some(path)
    } else {
        None
    }
}
