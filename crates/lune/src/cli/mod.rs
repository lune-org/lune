use std::{env::args_os, ffi::OsString, iter::once, process::ExitCode};

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
        // TODO: Figure out if there is a better way to do this using clap ... ?
        // https://github.com/lune-org/lune/issues/253
        if args_os()
            .nth(1)
            .is_some_and(|arg| arg.eq_ignore_ascii_case("run"))
        {
            // Options for lune come before the script path,
            // and all arguments after it are passed to the script
            let Some(script_index) = args_os()
                .skip(2)
                .position(|arg| !arg.as_encoded_bytes().starts_with(b"--"))
            else {
                return Self::parse(); // Will fail and return the help message
            };

            let mut command = RunCommand::parse_from(
                once(OsString::from("lune run")).chain(args_os().skip(2).take(script_index + 1)),
            );

            command.script_args = args_os()
                .skip(script_index + 3)
                .filter_map(|arg| arg.to_str().map(String::from))
                .collect::<Vec<_>>();

            Self {
                subcommand: Some(CliSubcommand::Run(command)),
            }
        } else {
            Self::parse()
        }
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
