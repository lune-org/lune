use std::{env, io::stdin, process::ExitCode};

use anyhow::{Context, Result};
use async_fs::read as read_to_vec;
use blocking::Unblock;
use clap::Parser;
use futures_lite::prelude::*;

use lune::Runtime;

use super::utils::files::{discover_script_path_including_lune_dirs, strip_shebang};

/// Run a script
#[derive(Debug, Clone, Parser)]
pub struct RunCommand {
    /// Script name or full path to the file to run
    pub(super) script_path: String,
    /// Arguments to pass to the script, stored in process.args
    pub(super) script_args: Vec<String>,
}

impl RunCommand {
    pub async fn run(self) -> Result<ExitCode> {
        // Figure out if we should read from stdin or from a file,
        // reading from stdin is marked by passing a single "-"
        // (dash) as the script name to run to the cli
        let is_stdin = &self.script_path == "-";
        let (script_display_name, script_contents) = if is_stdin {
            let mut stdin_contents = Vec::new();
            Unblock::new(stdin())
                .read_to_end(&mut stdin_contents)
                .await
                .context("Failed to read script contents from stdin")?;
            ("stdin".to_string(), stdin_contents)
        } else {
            let file_path = discover_script_path_including_lune_dirs(&self.script_path)?;
            let file_contents = read_to_vec(&file_path).await?;
            (file_path.display().to_string(), file_contents)
        };

        // Check if the user has explicitly disabled JIT (on by default)
        let jit_disabled = env::var("LUNE_LUAU_JIT")
            .ok()
            .is_some_and(|s| matches!(s.as_str(), "0" | "false" | "off"));

        // Create a new lune runtime with all globals & run the script
        let result = Runtime::new()?
            .with_args(self.script_args)
            .with_jit(!jit_disabled)
            .run_file(&script_display_name, strip_shebang(script_contents))
            .await;

        Ok(match result {
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
            Ok(values) => ExitCode::from(values.status()),
        })
    }
}
