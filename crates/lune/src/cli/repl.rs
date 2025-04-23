use std::{path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};
use clap::Parser;
use directories::UserDirs;
use rustyline::{error::ReadlineError, DefaultEditor};

use lune::Runtime;

const MESSAGE_WELCOME: &str = concat!("Lune v", env!("CARGO_PKG_VERSION"));
const MESSAGE_INTERRUPT: &str = "Interrupt: ^C again to exit";

enum PromptState {
    Regular,
    Continuation,
}

/// Launch an interactive REPL (default)
#[derive(Debug, Clone, Default, Parser)]
pub struct ReplCommand {}

impl ReplCommand {
    pub async fn run(self) -> Result<ExitCode> {
        println!("{MESSAGE_WELCOME}");

        let history_file_path: &PathBuf = &UserDirs::new()
            .context("Failed to find user home directory")?
            .home_dir()
            .join(".lune_history");
        if !history_file_path.exists() {
            tokio::fs::write(history_file_path, &[]).await?;
        }

        let mut repl = DefaultEditor::new()?;
        repl.load_history(history_file_path)?;

        let mut interrupt_counter = 0;
        let mut prompt_state = PromptState::Regular;
        let mut source_code = String::new();

        let mut lune_instance = Runtime::new();

        loop {
            let prompt = match prompt_state {
                PromptState::Regular => "> ",
                PromptState::Continuation => ">> ",
            };

            match repl.readline(prompt) {
                Ok(code) => {
                    interrupt_counter = 0;

                    // TODO: Should we add history entries for each separate line?
                    // Or should we add and save history only when we have complete
                    // lua input that may or may not be multiple lines long?
                    repl.add_history_entry(&code)?;
                    repl.save_history(history_file_path)?;

                    match prompt_state {
                        PromptState::Regular => source_code = code,
                        PromptState::Continuation => source_code.push_str(&code),
                    }
                }

                Err(ReadlineError::Eof) => break,
                Err(ReadlineError::Interrupted) => {
                    interrupt_counter += 1;

                    // NOTE: We actually want the user to do ^C twice to exit,
                    // and if we get an interrupt we should continue to the next
                    // readline loop iteration so we don't run input code twice
                    if interrupt_counter == 1 {
                        println!("{MESSAGE_INTERRUPT}");
                        continue;
                    }

                    break;
                }

                Err(err) => {
                    eprintln!("REPL ERROR: {err}");
                    return Ok(ExitCode::FAILURE);
                }
            }

            // TODO: Preserve context here somehow?
            let eval_result = lune_instance.run("REPL", &source_code).await;

            match eval_result {
                Ok(_) => prompt_state = PromptState::Regular,

                Err(err) => {
                    if err.is_incomplete_input() {
                        prompt_state = PromptState::Continuation;
                        source_code.push('\n');
                    } else {
                        eprintln!("{err}");
                    }
                }
            }
        }

        repl.save_history(history_file_path)?;

        Ok(ExitCode::SUCCESS)
    }
}
