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

#[cfg(test)]
mod tests {
    macro_rules! tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[tokio::test]
                async fn $name() {
                    let args = vec!["Foo".to_owned(), "Bar".to_owned()];
                    let cli = crate::Cli::from_path_with_args($value, args);
                    if let Err(e) = cli.run().await {
                        panic!("{}", e.to_string())
                    }
                }
            )*
        }
    }

    tests! {
        process_args: "tests/process/args",
        process_env: "tests/process/env",
        process_spawn: "tests/process/spawn",
    }
}
