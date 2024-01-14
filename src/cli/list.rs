use std::{fmt::Write as _, process::ExitCode};

use anyhow::Result;
use clap::Parser;

use super::utils::listing::{find_lune_scripts, sort_lune_scripts, write_lune_scripts_list};

/// List scripts available to run
#[derive(Debug, Clone, Parser)]
pub struct ListCommand {}

impl ListCommand {
    pub async fn run(self) -> Result<ExitCode> {
        let sorted_relative = find_lune_scripts(false).await.map(sort_lune_scripts);

        let sorted_home_dir = find_lune_scripts(true).await.map(sort_lune_scripts);
        if sorted_relative.is_err() && sorted_home_dir.is_err() {
            eprintln!("{}", sorted_relative.unwrap_err());
            return Ok(ExitCode::FAILURE);
        }

        let sorted_relative = sorted_relative.unwrap_or(Vec::new());
        let sorted_home_dir = sorted_home_dir.unwrap_or(Vec::new());

        let mut buffer = String::new();
        if !sorted_relative.is_empty() {
            if sorted_home_dir.is_empty() {
                write!(&mut buffer, "Available scripts:")?;
            } else {
                write!(&mut buffer, "Available scripts in current directory:")?;
            }
            write_lune_scripts_list(&mut buffer, sorted_relative)?;
        }
        if !sorted_home_dir.is_empty() {
            write!(&mut buffer, "Available global scripts:")?;
            write_lune_scripts_list(&mut buffer, sorted_home_dir)?;
        }

        if buffer.is_empty() {
            println!("No scripts found.");
        } else {
            print!("{buffer}");
        }

        Ok(ExitCode::SUCCESS)
    }
}
