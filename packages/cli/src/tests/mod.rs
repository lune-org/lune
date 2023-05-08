use anyhow::Result;

use crate::cli::Cli;

mod bin_dir;
mod file_checks;
mod file_type;
mod run_cli;

pub(crate) use run_cli::*;

#[tokio::test]
async fn list() -> Result<()> {
    Cli::new().list().run().await?;
    Ok(())
}

#[tokio::test]
async fn generate_typedef_files() -> Result<()> {
    run_cli(Cli::new().setup()).await?;
    // TODO: Implement test
    Ok(())
}
