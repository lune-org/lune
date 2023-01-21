#![deny(clippy::all, clippy::cargo, clippy::pedantic)]
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

#[tokio::test]
async fn hello_lune() {
    let args = vec!["Hello, test! ✅".to_owned()];
    let cli = Cli::from_path_with_args("hello_lune", args);
    let result = cli.run().await;
    assert!(result.is_ok());
}
