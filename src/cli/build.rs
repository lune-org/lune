use std::{
    env::consts,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use directories::BaseDirs;
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::{fs, io::AsyncWriteExt, task::spawn_blocking};

use crate::standalone::metadata::{Metadata, CURRENT_EXE};

const TARGET_BASE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    BaseDirs::new()
        .unwrap()
        .home_dir()
        .to_path_buf()
        .join(".lune")
        .join("target")
        .join(env!("CARGO_PKG_VERSION"))
});

/// Build a standalone executable
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
}

impl BuildCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let output_path = self
            .output
            .unwrap_or_else(|| self.input.with_extension(consts::EXE_EXTENSION));

        let input_path_displayed = self.input.display();

        // Try to read the input file
        let source_code = fs::read(&self.input)
            .await
            .context("failed to read input file")?;

        // Dynamically derive the base executable path based on the CLI arguments provided
        let (base_exe_path, output_path) = get_base_exe_path(self.target, output_path).await?;

        // Read the contents of the lune interpreter as our starting point
        println!(
            "Compiling standalone binary from {}",
            style(input_path_displayed).green()
        );
        let patched_bin = Metadata::create_env_patched_bin(base_exe_path, source_code.clone())
            .await
            .context("failed to create patched binary")?;

        // And finally write the patched binary to the output file
        println!(
            "Writing standalone binary to {}",
            style(output_path.display()).blue()
        );
        write_executable_file_to(output_path, patched_bin).await?; // Read & execute for all, write for owner

        Ok(ExitCode::SUCCESS)
    }
}

async fn write_executable_file_to(
    path: impl AsRef<Path>,
    bytes: impl AsRef<[u8]>,
) -> Result<(), std::io::Error> {
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

/// Errors that may occur when building a standalone binary
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("failed to find lune target '{0}' in GitHub release")]
    ReleaseTargetNotFound(String),
    #[error("failed to find lune binary '{0}' in downloaded zip file")]
    ZippedBinaryNotFound(String),
    #[error("failed to download lune binary: {0}")]
    Download(#[from] reqwest::Error),
    #[error("failed to unzip lune binary: {0}")]
    Unzip(#[from] zip_next::result::ZipError),
    #[error("panicked while unzipping lune binary: {0}")]
    UnzipJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type BuildResult<T, E = BuildError> = std::result::Result<T, E>;

/// Discovers the path to the base executable to use for cross-compilation
async fn get_base_exe_path(
    target: Option<String>,
    output_path: PathBuf,
) -> BuildResult<(PathBuf, PathBuf)> {
    if let Some(target_inner) = target {
        let current_target = format!("{}-{}", consts::OS, consts::ARCH);

        let target_exe_extension = match target_inner.as_str() {
            "windows-x86_64" => "exe",
            _ => "",
        };

        if target_inner == current_target {
            // If the target is the host target, just use the current executable
            return Ok((
                CURRENT_EXE.to_path_buf(),
                output_path.with_extension(consts::EXE_EXTENSION),
            ));
        }

        let path = TARGET_BASE_DIR.join(format!("lune-{target_inner}.{target_exe_extension}"));

        // Create the target base directory in the lune home if it doesn't already exist
        if !TARGET_BASE_DIR.exists() {
            fs::create_dir_all(TARGET_BASE_DIR.to_path_buf()).await?;
        }

        // If a cached target base executable doesn't exist, attempt to download it
        if !path.exists() {
            println!("Requested target does not exist in cache and must be downloaded");
            download_target_to_cache(target_inner, target_exe_extension, &path).await?;
        }

        Ok((path, output_path.with_extension(target_exe_extension)))
    } else {
        // If the target flag was not specified, just use the current executable
        Ok((
            CURRENT_EXE.to_path_buf(),
            output_path.with_extension(consts::EXE_EXTENSION),
        ))
    }
}

/// Downloads the target base executable to the cache directory
async fn download_target_to_cache(
    target: String,
    target_exe_extension: &str,
    path: &PathBuf,
) -> BuildResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let target_triple = format!("lune-{version}-{target}");

    let release_url = format!(
        "{base_url}/v{version}/{target_triple}.zip",
        base_url = "https://github.com/lune-org/lune/releases/download",
    );
    println!("Downloading {target_triple}");

    // Try to request to download the zip file from the target url,
    // making sure transient errors are handled gracefully and
    // with a different error message than "not found"
    let response = reqwest::get(release_url).await?;
    if !response.status().is_success() {
        if response.status().as_u16() == 404 {
            return Err(BuildError::ReleaseTargetNotFound(target));
        }
        return Err(BuildError::Download(
            response.error_for_status().unwrap_err(),
        ));
    }

    // Receive the full zip file
    let zip_bytes = response.bytes().await?.to_vec();
    let zip_file = Cursor::new(zip_bytes);

    // Look for and extract the binary file from the zip file
    let binary_file_name = format!(
        "lune{}{target_exe_extension}",
        if target_exe_extension.is_empty() {
            ""
        } else {
            "."
        }
    );

    // NOTE: We use spawn_blocking here since reading a
    // zip archive is a somewhat slow / blocking operation
    let binary_file_handle = spawn_blocking(move || {
        let mut archive = zip_next::ZipArchive::new(zip_file)?;

        let mut binary = Vec::new();
        archive
            .by_name(&binary_file_name)
            .or(Err(BuildError::ZippedBinaryNotFound(binary_file_name)))?
            .read_to_end(&mut binary)?;

        Ok::<_, BuildError>(binary)
    });
    let binary_file_contents = binary_file_handle.await??;

    // Finally write the decompressed data to the target base directory
    write_executable_file_to(&path, binary_file_contents).await?;

    println!("Downloaded {target_triple} successfully");

    Ok(())
}
