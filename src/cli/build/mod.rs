use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use thiserror::Error;
use tokio::{fs, io::AsyncWriteExt, task::spawn_blocking};

use crate::standalone::metadata::{Metadata, CURRENT_EXE};

mod target;

use self::target::{Target, CACHE_DIR};

/// Build a standalone executable
#[derive(Debug, Clone, Parser)]
pub struct BuildCommand {
    /// The path to the input file
    pub input: PathBuf,

    /// The path to the output file - defaults to the
    /// input file path with an executable extension
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// The target to compile for in the format `os-arch` -
    /// defaults to the os and arch of the current system
    #[clap(short, long)]
    pub target: Option<Target>,
}

impl BuildCommand {
    pub async fn run(self) -> Result<ExitCode> {
        // Derive target spec to use, or default to the current host system
        let target = self.target.unwrap_or_else(Target::current_system);

        // Derive paths to use, and make sure the output path is
        // not the same as the input, so that we don't overwrite it
        let output_path = self
            .output
            .clone()
            .unwrap_or_else(|| remove_source_file_ext(&self.input));
        let output_path = output_path.with_extension(target.exe_extension());
        if output_path == self.input {
            if self.output.is_some() {
                bail!("output path cannot be the same as input path");
            }
            bail!("output path cannot be the same as input path, please specify a different output path");
        }

        // Try to read the input file
        let source_code = fs::read(&self.input)
            .await
            .context("failed to read input file")?;

        // Derive the base executable path based on the arguments provided
        let base_exe_path = get_or_download_base_executable(target).await?;

        // Read the contents of the lune interpreter as our starting point
        println!(
            "Compiling standalone binary from {}",
            style(self.input.display()).green()
        );
        let patched_bin = Metadata::create_env_patched_bin(base_exe_path, source_code)
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

/// Removes the source file extension from the given path, if it has one
/// A source file extension is an extension such as `.lua` or `.luau`
pub fn remove_source_file_ext(path: &Path) -> PathBuf {
    if path
        .extension()
        .is_some_and(|ext| matches!(ext.to_str(), Some("lua" | "luau")))
    {
        path.with_extension("")
    } else {
        path.to_path_buf()
    }
}

/// Writes the given bytes to a file at the specified path, and makes sure it has permissions to be executed
pub async fn write_executable_file_to(
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
    ReleaseTargetNotFound(Target),
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

/// Discovers the path to the base executable to use for cross-compilation, and downloads it if necessary
pub async fn get_or_download_base_executable(target: Target) -> BuildResult<PathBuf> {
    // If the target matches the current system, just use the current executable
    if target.is_current_system() {
        return Ok(CURRENT_EXE.to_path_buf());
    }

    // If a cached target base executable doesn't exist, attempt to download it
    if !target.cache_path().exists() {
        return Ok(target.cache_path());
    }

    // The target is not cached, we must download it
    println!("Requested target '{target}' does not exist in cache");
    let version = env!("CARGO_PKG_VERSION");
    let target_triple = format!("lune-{version}-{target}");

    let release_url = format!(
        "{base_url}/v{version}/{target_triple}.zip",
        base_url = "https://github.com/lune-org/lune/releases/download",
    );

    // NOTE: This is not entirely accurate, but it is clearer for a user
    println!("Downloading {target_triple}{}...", target.exe_suffix());

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
    // NOTE: We use spawn_blocking here since reading a zip
    // archive is a somewhat slow / blocking operation
    let binary_file_name = format!("lune{}", target.exe_suffix());
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

    // Finally, write the extracted binary to the cache
    if !CACHE_DIR.exists() {
        fs::create_dir_all(CACHE_DIR.as_path()).await?;
    }
    write_executable_file_to(target.cache_path(), binary_file_contents).await?;
    println!("Downloaded successfully and added to cache");

    Ok(target.cache_path())
}
