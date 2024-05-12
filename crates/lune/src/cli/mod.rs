use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub(crate) mod build;
pub(crate) mod list;
pub(crate) mod repl;
pub(crate) mod run;
pub(crate) mod setup;
pub(crate) mod utils;

pub use self::{
    build::BuildCommand, list::ListCommand, repl::ReplCommand, run::RunCommand, setup::SetupCommand,
};

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Run(RunCommand),
    List(ListCommand),
    Setup(SetupCommand),
    Build(BuildCommand),
    Repl(ReplCommand),
}

impl Default for CliSubcommand {
    fn default() -> Self {
        Self::Repl(ReplCommand::default())
    }
}

/// Lune, a standalone Luau runtime
#[derive(Parser, Debug, Default, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: Option<CliSubcommand>,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn run(self) -> Result<ExitCode> {
        match self.subcommand.unwrap_or_default() {
            CliSubcommand::Run(cmd) => cmd.run().await,
            CliSubcommand::List(cmd) => cmd.run().await,
            CliSubcommand::Setup(cmd) => cmd.run().await,
            CliSubcommand::Build(cmd) => cmd.run().await,
            CliSubcommand::Repl(cmd) => cmd.run().await,
        }
    }
}
