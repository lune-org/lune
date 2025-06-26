use std::{env, io::stdin, process::ExitCode};

use anyhow::{Context, Result};
use blocking::Unblock;
use clap::Parser;
use futures_lite::prelude::*;

use lune::Runtime;

use super::utils::files::discover_script_path_including_lune_dirs;

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
        // Check if the user has explicitly disabled JIT (on by default)
        let jit_disabled = env::var("LUNE_LUAU_JIT")
            .ok()
            .is_some_and(|s| matches!(s.as_str(), "0" | "false" | "off"));

        // Create a new lune runtime with all globals & run the script
        let mut rt = Runtime::new()?
            .with_args(self.script_args)
            .with_jit(!jit_disabled);

        // Figure out if we should run stdin or run a file,
        // reading from stdin is marked by passing a single "-"
        // (dash) as the script name to run to the cli
        let result = if &self.script_path == "-" {
            let mut stdin_contents = Vec::new();
            Unblock::new(stdin())
                .read_to_end(&mut stdin_contents)
                .await
                .context("Failed to read script contents from stdin")?;
            rt.run_custom("stdin", stdin_contents).await
        } else {
            let file_path = discover_script_path_including_lune_dirs(&self.script_path)?;
            rt.run_file(file_path).await
        };

        Ok(match result {
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
            Ok(values) => ExitCode::from(values.status()),
        })
    }
}
