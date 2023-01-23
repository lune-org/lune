#![deny(clippy::all)]
#![warn(clippy::cargo, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value, clippy::match_bool)]

use anyhow::Result;
use clap::Parser;

mod cli;
mod utils;

use cli::Cli;

fn main() -> Result<()> {
    smol::block_on(async {
        let cli = Cli::parse();
        cli.run().await?;
        Ok(())
    })
}
