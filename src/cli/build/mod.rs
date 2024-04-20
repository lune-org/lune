use std::{path::PathBuf, process::ExitCode};

use anyhow::{bail, Context, Result};
use clap::Parser;
use console::style;
use tokio::fs;

use crate::standalone::metadata::Metadata;

mod base_exe;
mod files;
mod result;
mod target;

use self::base_exe::get_or_download_base_executable;
use self::files::{remove_source_file_ext, write_executable_file_to};
use self::target::BuildTarget;

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
    pub target: Option<BuildTarget>,
}

impl BuildCommand {
    pub async fn run(self) -> Result<ExitCode> {
        // Derive target spec to use, or default to the current host system
        let target = self.target.unwrap_or_else(BuildTarget::current_system);

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
