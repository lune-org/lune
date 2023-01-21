use std::fs::read_to_string;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use lune::Lune;

use crate::utils::{files::find_parse_file_path, github::Client as GithubClient};

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

impl Cli {
    #[allow(dead_code)]
    pub fn from_path<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            script_path: Some(path.into()),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
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

    pub async fn run(self) -> Result<()> {
        // Download definition files, if wanted
        let download_types_requested = self.download_selene_types || self.download_luau_types;
        if download_types_requested {
            let client = GithubClient::new().map_err(mlua::Error::external)?;
            let release = client
                .fetch_release_for_this_version()
                .await
                .map_err(mlua::Error::external)?;
            if self.download_selene_types {
                println!("Downloading Selene type definitions...");
                client
                    .fetch_release_asset(&release, "lune.yml")
                    .await
                    .map_err(mlua::Error::external)?;
            }
            if self.download_luau_types {
                println!("Downloading Luau type definitions...");
                client
                    .fetch_release_asset(&release, "luneTypes.d.luau")
                    .await
                    .map_err(mlua::Error::external)?;
            }
        }
        if self.script_path.is_none() {
            // Only downloading types without running a script is completely
            // fine, and we should just exit the program normally afterwards
            if download_types_requested {
                return Ok(());
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
        Lune::new()?
            .with_args(self.script_args)?
            .with_default_globals()?
            .run_with_name(&file_contents, &file_display_name)
            .await?;
        Ok(())
    }
}
