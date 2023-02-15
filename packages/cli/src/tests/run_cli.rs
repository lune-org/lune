use anyhow::Result;

use crate::cli::Cli;

use super::bin_dir::{enter_bin_dir, leave_bin_dir};

pub async fn run_cli(cli: Cli) -> Result<()> {
    enter_bin_dir().await?;
    cli.run().await?;
    leave_bin_dir()?;
    Ok(())
}
