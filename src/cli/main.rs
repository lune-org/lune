#![deny(clippy::all)]
#![warn(clippy::cargo, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value, clippy::match_bool)]

use anyhow::Result;
use clap::Parser;

mod cli;
mod utils;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await?;
    Ok(())
}
