#![deny(clippy::all)]
#![warn(clippy::cargo, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value, clippy::match_bool)]

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

mod cli;
mod utils;

use cli::Cli;

fn main() -> Result<ExitCode> {
    smol::block_on(async { Cli::parse().run().await })
}
