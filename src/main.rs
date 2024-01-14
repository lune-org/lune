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

pub(crate) mod cli;
pub(crate) mod standalone;

use cli::Cli;
use console::style;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .with_target(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_level(true)
        .init();

    if let Some(bin) = standalone::check().await {
        return standalone::run(bin).await.unwrap();
    }

    match Cli::new().run().await {
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
