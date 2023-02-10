#![deny(clippy::all)]
#![warn(clippy::cargo, clippy::pedantic)]
#![allow(
    clippy::cargo_common_metadata,
    clippy::match_bool,
    clippy::module_name_repetitions,
    clippy::multiple_crate_versions,
    clippy::needless_pass_by_value
)]

use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

pub(crate) mod cli;
pub(crate) mod gen;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    Cli::parse().run().await
}
