use std::{
    fmt::Write,
    path::{Path, PathBuf},
    process::{exit, ExitCode},
};

use anyhow::{Error, Result};
use clap::Command;
use directories::UserDirs;
use lune::Lune;
use rustyline::{error::ReadlineError, DefaultEditor};

#[derive(PartialEq)]
enum PromptState {
    Regular,
    Continuation,
}

// Isn't dependency injection plain awesome?!
pub async fn show_interface(cmd: Command) -> Result<ExitCode> {
    let lune_version = cmd.get_version();

    // The version is mandatory and will always exist
    println!("Lune v{}", lune_version.unwrap());

    let lune_instance = Lune::new();

    let mut repl = DefaultEditor::new()?;
    let history_file_path: &PathBuf = &UserDirs::new()
        .ok_or(Error::msg("cannot find user home directory"))?
        .home_dir()
        .join(".lune_history");

    if !history_file_path.exists() {
        std::fs::write(&history_file_path, String::new())?;
    }

    repl.load_history(history_file_path)?;

    let mut interrupt_counter = 0u32;
    let mut prompt_state: PromptState = PromptState::Regular;
    let mut source_code = String::new();

    loop {
        let prompt = match prompt_state {
            PromptState::Regular => "> ",
            PromptState::Continuation => ">> ",
        };

        match repl.readline(prompt) {
            Ok(code) => {
                if prompt_state == PromptState::Continuation {
                    write!(&mut source_code, "{}", code)?;
                } else if prompt_state == PromptState::Regular {
                    source_code = code.clone();
                }

                repl.add_history_entry(code.as_str())?;

                // If source code eval was requested, we reset the counter
                interrupt_counter = 0;
            }

            Err(ReadlineError::Interrupted) => {
                // HACK: We actually want the user to do ^C twice to exit,
                // but the user would need to ^C one more time even after
                // the check passes, so we check for 1 instead of 2
                if interrupt_counter != 1 {
                    println!("Interrupt: ^C again to exit");

                    // Increment the counter
                    interrupt_counter += 1;
                } else {
                    repl.save_history(history_file_path)?;
                    break;
                }
            }
            Err(ReadlineError::Eof) => {
                repl.save_history(history_file_path)?;
                break;
            }
            Err(err) => {
                eprintln!("REPL ERROR: {}", err.to_string());

                // This isn't a good way to exit imo, once again
                exit(1);
            }
        };

        let eval_result = lune_instance.run("REPL", source_code.clone()).await;

        match eval_result {
            Ok(_) => prompt_state = PromptState::Regular,

            Err(err) => {
                if err.is_incomplete_input() {
                    prompt_state = PromptState::Continuation;
                    source_code.push_str("\n")
                } else {
                    eprintln!("{}", err);
                }
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}
