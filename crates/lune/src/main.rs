#![allow(clippy::cargo_common_metadata)]

use std::{io::stderr, process::ExitCode};

#[cfg(feature = "cli")]
pub(crate) mod cli;
pub(crate) mod dirs;
pub(crate) mod standalone;

use lune_utils::fmt::Label;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .with_writer(stderr)
        .init();

    async_io::block_on(async {
        if let Some(bin) = standalone::check().await {
            return standalone::run(bin).await.unwrap();
        }

        #[cfg(feature = "cli")]
        {
            match cli::Cli::new().run().await {
                Ok(code) => code,
                Err(err) => {
                    eprintln!("{}\n{err:?}", Label::Error);
                    ExitCode::FAILURE
                }
            }
        }

        #[cfg(not(feature = "cli"))]
        {
            eprintln!("{}\nCLI feature is disabled", Label::Error);
            ExitCode::FAILURE
        }
    })
}
