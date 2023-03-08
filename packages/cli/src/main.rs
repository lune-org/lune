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

use clap::Parser;

pub(crate) mod cli;
pub(crate) mod gen;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use cli::Cli;
use console::style;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    match Cli::parse().run().await {
        Ok(code) => code,
        Err(err) => {
            eprintln!(
                "{}{}{}\n{err:?}",
                style("[").dim(),
                style("ERROR").red(),
                style("]").dim(),
            );
            ExitCode::FAILURE
        }
    }
}
