use anyhow::Result;

use crate::cli::{Cli, FILE_NAME_DOCS, FILE_NAME_LUAU_TYPES, FILE_NAME_SELENE_TYPES};

mod bin_dir;
mod file_checks;
mod file_type;
mod run_cli;

pub(crate) use file_checks::*;
pub(crate) use file_type::*;
pub(crate) use run_cli::*;

#[tokio::test]
async fn list() -> Result<()> {
    Cli::new().list().run().await?;
    Ok(())
}

#[tokio::test]
async fn generate_selene_types() -> Result<()> {
    run_cli(Cli::new().generate_selene_types()).await?;
    ensure_file_exists_and_is(FILE_NAME_SELENE_TYPES, FileType::Yaml).await?;
    Ok(())
}

#[tokio::test]
async fn generate_luau_types() -> Result<()> {
    run_cli(Cli::new().generate_luau_types()).await?;
    ensure_file_exists_and_is(FILE_NAME_LUAU_TYPES, FileType::Luau).await?;
    Ok(())
}

#[tokio::test]
async fn generate_docs_file() -> Result<()> {
    run_cli(Cli::new().generate_docs_file()).await?;
    ensure_file_exists_and_is(FILE_NAME_DOCS, FileType::Json).await?;
    Ok(())
}
