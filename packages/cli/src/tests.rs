use std::env::{current_dir, set_current_dir};

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::fs::{create_dir_all, read_to_string, remove_file};

use crate::cli::{Cli, LUNE_LUAU_FILE_NAME, LUNE_SELENE_FILE_NAME};

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
