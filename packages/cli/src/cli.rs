use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use lune::Lune;
use tokio::fs::{read_to_string, write};

use crate::{
    gen::generate_docs_json_from_definitions,
    utils::{
        files::find_parse_file_path,
        github::Client as GithubClient,
        listing::{find_lune_scripts, print_lune_scripts, sort_lune_scripts},
    },
};

const LUNE_SELENE_FILE_NAME: &str = "lune.yml";
const LUNE_LUAU_FILE_NAME: &str = "luneTypes.d.luau";
const LUNE_DOCS_FILE_NAME: &str = "luneDocs.json";

/// Lune CLI
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Path to the file to run, or the name
    /// of a luau file in a lune directory
    ///
    /// Can be omitted when downloading type definitions
    script_path: Option<String>,
    /// Arguments to pass to the file as vararg (...)
    script_args: Vec<String>,
    /// Pass this flag to list scripts inside of
    /// nearby `lune` and / or `.lune` directories
    #[clap(long, short = 'l')]
    list: bool,
    /// Pass this flag to download the Selene type
    /// definitions file to the current directory
    #[clap(long)]
    download_selene_types: bool,
    /// Pass this flag to download the Luau type
    /// definitions file to the current directory
    #[clap(long)]
    download_luau_types: bool,
    /// Pass this flag to generate the Lune documentation file
    /// from a luau type definitions file in the current directory
    #[clap(long)]
    generate_docs_file: bool,
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

    pub fn list() -> Self {
        Self {
            list: true,
            ..Default::default()
        }
    }

    pub async fn run(self) -> Result<ExitCode> {
        // List files in `lune` and `.lune` directories, if wanted
        // This will also exit early and not run anything else
        if self.list {
            match find_lune_scripts().await {
                Ok(scripts) => {
                    let sorted = sort_lune_scripts(scripts);
                    if sorted.is_empty() {
                        println!("No scripts found.");
                    } else {
                        print!("Available scripts:");
                        print_lune_scripts(sorted)?;
                    }
                    return Ok(ExitCode::SUCCESS);
                }
                Err(e) => {
                    eprintln!("{e}");
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
        // Download definition files, if wanted
        let download_types_requested = self.download_selene_types || self.download_luau_types;
        if download_types_requested {
            let client = GithubClient::new();
            let release = client.fetch_release_for_this_version().await?;
            if self.download_selene_types {
                println!("Downloading Selene type definitions...");
                client
                    .fetch_release_asset(&release, LUNE_SELENE_FILE_NAME)
                    .await?;
            }
            if self.download_luau_types {
                println!("Downloading Luau type definitions...");
                client
                    .fetch_release_asset(&release, LUNE_LUAU_FILE_NAME)
                    .await?;
            }
        }
        // Generate docs file, if wanted
        if self.generate_docs_file {
            let defs_contents = read_to_string(LUNE_LUAU_FILE_NAME).await?;
            let docs_root = generate_docs_json_from_definitions(&defs_contents, "roblox/global")?;
            let docs_contents = serde_json::to_string_pretty(&docs_root)?;
            write(LUNE_DOCS_FILE_NAME, &docs_contents).await?;
        }
        if self.script_path.is_none() {
            // Only downloading types without running a script is completely
            // fine, and we should just exit the program normally afterwards
            // Same thing goes for generating the docs file
            if download_types_requested || self.generate_docs_file {
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
        let file_contents = read_to_string(&file_path).await?;
        // Display the file path relative to cwd with no extensions in stack traces
        let file_display_name = file_path.with_extension("").display().to_string();
        // Create a new lune object with all globals & run the script
        let lune = Lune::new().with_args(self.script_args).with_all_globals();
        let result = lune.run(&file_display_name, &file_contents).await;
        Ok(match result {
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
            Ok(code) => code,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env::{current_dir, set_current_dir};

    use anyhow::{bail, Context, Result};
    use serde_json::Value;
    use tokio::fs::{create_dir_all, read_to_string, remove_file};

    use super::{Cli, LUNE_LUAU_FILE_NAME, LUNE_SELENE_FILE_NAME};

    async fn run_cli(cli: Cli) -> Result<()> {
        let path = current_dir()
            .context("Failed to get current dir")?
            .join("bin");
        create_dir_all(&path)
            .await
            .context("Failed to create bin dir")?;
        set_current_dir(&path).context("Failed to set current dir")?;
        cli.run().await?;
        Ok(())
    }

    async fn ensure_file_exists_and_is_not_json(file_name: &str) -> Result<()> {
        match read_to_string(file_name)
            .await
            .context("Failed to read definitions file")
        {
            Ok(file_contents) => match serde_json::from_str::<Value>(&file_contents) {
                Err(_) => {
                    remove_file(file_name)
                        .await
                        .context("Failed to remove definitions file")?;
                    Ok(())
                }
                Ok(_) => bail!("Downloading selene definitions returned json, expected luau"),
            },
            Err(e) => bail!("Failed to download selene definitions!\n{e}"),
        }
    }

    #[tokio::test]
    async fn list() -> Result<()> {
        Cli::list().run().await?;
        Ok(())
    }

    #[tokio::test]
    async fn download_selene_types() -> Result<()> {
        run_cli(Cli::download_selene_types()).await?;
        ensure_file_exists_and_is_not_json(LUNE_SELENE_FILE_NAME).await?;
        Ok(())
    }

    #[tokio::test]
    async fn download_luau_types() -> Result<()> {
        run_cli(Cli::download_luau_types()).await?;
        ensure_file_exists_and_is_not_json(LUNE_LUAU_FILE_NAME).await?;
        Ok(())
    }
}
