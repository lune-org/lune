use std::{path::PathBuf, process::ExitCode};

use anyhow::{Context, Result, bail};
use async_fs as fs;
use clap::Parser;
use console::style;

use crate::standalone::metadata::Metadata;

mod base_exe;
mod files;
mod result;
mod target;

use self::base_exe::get_or_download_base_executable;
use self::files::{remove_source_file_ext, write_executable_file_to};
use self::target::{BuildTarget, BuildTargetOS};

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
        let mut output_path = output_path;
        if target.os == BuildTargetOS::Windows {
            let has_exe_ext = output_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
            if !has_exe_ext {
                let mut os_string = output_path.into_os_string();
                os_string.push(".exe");
                output_path = PathBuf::from(os_string);
            }
        }
        if output_path == self.input {
            if self.output.is_some() {
                bail!("output path cannot be the same as input path");
            }
            bail!(
                "output path cannot be the same as input path, please specify a different output path"
            );
        }

        // Try to read the given input file
        // FUTURE: We should try and resolve a full require file graph using the input
        // path here instead, see the notes in the `standalone` module for more details
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::build::target::{BuildTargetArch, BuildTargetOS};
    use std::path::Path;

    #[test]
    fn test_output_path_handling() {
        let windows_target = BuildTarget {
            os: BuildTargetOS::Windows,
            arch: BuildTargetArch::X86_64,
        };
        let unix_target = BuildTarget {
            os: BuildTargetOS::Linux,
            arch: BuildTargetArch::X86_64,
        };

        // Case 1: Windows target, no .exe extension
        let p = PathBuf::from("SomeTool-v1.2.3-windows-x86_64");
        let mut output_path = p;
        if windows_target.os == BuildTargetOS::Windows {
            let has_exe_ext = output_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
            if !has_exe_ext {
                let mut os_string = output_path.into_os_string();
                os_string.push(".exe");
                output_path = PathBuf::from(os_string);
            }
        }
        assert_eq!(output_path, Path::new("SomeTool-v1.2.3-windows-x86_64.exe"));

        // Case 2: Windows target, already has .exe extension
        let p = PathBuf::from("SomeTool-v1.2.3-windows-x86_64.exe");
        let mut output_path = p;
        if windows_target.os == BuildTargetOS::Windows {
            let has_exe_ext = output_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
            if !has_exe_ext {
                let mut os_string = output_path.into_os_string();
                os_string.push(".exe");
                output_path = PathBuf::from(os_string);
            }
        }
        assert_eq!(output_path, Path::new("SomeTool-v1.2.3-windows-x86_64.exe"));

        // Case 3: Unix target, has dot in filename
        let p = PathBuf::from("SomeTool-v1.2.3-linux-x86_64");
        let mut output_path = p;
        if unix_target.os == BuildTargetOS::Windows {
            let has_exe_ext = output_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
            if !has_exe_ext {
                let mut os_string = output_path.into_os_string();
                os_string.push(".exe");
                output_path = PathBuf::from(os_string);
            }
        }
        assert_eq!(output_path, Path::new("SomeTool-v1.2.3-linux-x86_64"));
    }
}
