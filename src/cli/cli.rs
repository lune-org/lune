use std::{fs::read_to_string, process::ExitCode};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use mlua::prelude::*;

use lune::Lune;

use crate::utils::{files::find_parse_file_path, github::Client as GithubClient};

const LUNE_SELENE_FILE_NAME: &str = "lune.yml";
const LUNE_LUAU_FILE_NAME: &str = "luneTypes.d.luau";

/// Lune CLI
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the file to run, or the name
    /// of a luau file in a lune directory
    ///
    /// Can be omitted when downloading type definitions
    script_path: Option<String>,
    /// Arguments to pass to the file as vararg (...)
    script_args: Vec<String>,
    /// Pass this flag to download the Selene type
    /// definitions file to the current directory
    #[clap(long)]
    download_selene_types: bool,
    /// Pass this flag to download the Luau type
    /// definitions file to the current directory
    #[clap(long)]
    download_luau_types: bool,
}

#[allow(dead_code)]
impl Cli {
    pub fn from_path<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            script_path: Some(path.into()),
            ..Default::default()
        }
    }

    pub fn from_path_with_args<S, A>(path: S, args: A) -> Self
    where
        S: Into<String>,
        A: Into<Vec<String>>,
    {
        Self {
            script_path: Some(path.into()),
            script_args: args.into(),
            ..Default::default()
        }
    }

    pub fn download_selene_types() -> Self {
        Self {
            download_selene_types: true,
            ..Default::default()
        }
    }

    pub fn download_luau_types() -> Self {
        Self {
            download_luau_types: true,
            ..Default::default()
        }
    }

    pub async fn run(self) -> Result<ExitCode> {
        // Download definition files, if wanted
        let download_types_requested = self.download_selene_types || self.download_luau_types;
        if download_types_requested {
            let client = GithubClient::new();
            let release = client
                .fetch_release_for_this_version()
                .await
                .map_err(LuaError::external)?;
            if self.download_selene_types {
                println!("Downloading Selene type definitions...");
                client
                    .fetch_release_asset(&release, LUNE_SELENE_FILE_NAME)
                    .await
                    .map_err(LuaError::external)?;
            }
            if self.download_luau_types {
                println!("Downloading Luau type definitions...");
                client
                    .fetch_release_asset(&release, LUNE_LUAU_FILE_NAME)
                    .await
                    .map_err(LuaError::external)?;
            }
        }
        if self.script_path.is_none() {
            // Only downloading types without running a script is completely
            // fine, and we should just exit the program normally afterwards
            if download_types_requested {
                return Ok(ExitCode::SUCCESS);
            }
            // HACK: We know that we didn't get any arguments here but since
            // script_path is optional clap will not error on its own, to fix
            // we will duplicate the cli command and make arguments required,
            // which will then fail and print out the normal help message
            let cmd = Cli::command();
            cmd.arg_required_else_help(true).get_matches();
        }
        // Parse and read the wanted file
        let file_path = find_parse_file_path(&self.script_path.unwrap())?;
        let file_contents = read_to_string(&file_path)?;
        // Display the file path relative to cwd with no extensions in stack traces
        let file_display_name = file_path.with_extension("").display().to_string();
        // Create a new lune object with all globals & run the script
        let lune = Lune::new().with_args(self.script_args).with_all_globals();
        let result = lune.run(&file_display_name, &file_contents).await;
        Ok(match result {
            Err(e) => {
                eprintln!("{e}");
                ExitCode::from(1)
            }
            Ok(code) => code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, LUNE_LUAU_FILE_NAME, LUNE_SELENE_FILE_NAME};
    use anyhow::{bail, Result};
    use serde_json::Value;
    use smol::fs::{create_dir_all, read_to_string};
    use std::env::set_current_dir;

    async fn run_cli(cli: Cli) -> Result<()> {
        create_dir_all("bin").await?;
        set_current_dir("bin")?;
        cli.run().await?;
        Ok(())
    }

    #[test]
    fn download_selene_types() -> Result<()> {
        smol::block_on(async {
            run_cli(Cli::download_selene_types()).await?;
            match read_to_string(LUNE_SELENE_FILE_NAME).await {
                Ok(file_contents) => match serde_json::from_str::<Value>(&file_contents) {
                    Err(_) => Ok(()),
                    Ok(_) => bail!("Downloading selene definitions returned json, expected luau"),
                },
                Err(_) => bail!("Failed to download selene definitions!"),
            }
        })
    }

    #[test]
    fn download_luau_types() -> Result<()> {
        smol::block_on(async {
            run_cli(Cli::download_luau_types()).await?;
            match read_to_string(LUNE_LUAU_FILE_NAME).await {
                Ok(file_contents) => match serde_json::from_str::<Value>(&file_contents) {
                    Err(_) => Ok(()),
                    Ok(_) => bail!("Downloading luau definitions returned json, expected luau"),
                },
                Err(_) => bail!("Failed to download luau definitions!"),
            }
        })
    }
}
