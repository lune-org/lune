use std::{
    io::{Error, ErrorKind},
    path::PathBuf,
    process::{exit, ExitCode},
};

use clap::Command;
use lune::{Lune, LuneError};
use rustyline::{error::ReadlineError, DefaultEditor};

use super::Cli;

// Isn't dependency injection plain awesome?!
pub async fn show_interface(cmd: Command) -> Result<ExitCode, anyhow::Error> {
    let lune_version = cmd.get_version();

    // The version is mandatory and will always exist
    println!("Lune v{}", lune_version.unwrap());

    let lune_instance = Lune::new();

    let mut repl = DefaultEditor::new()?;

    match repl.load_history(&(|| -> PathBuf {
        let dir_opt = home::home_dir();

        if let Some(dir) = dir_opt {
            dir.join(".lune_history")
        } else {
            eprintln!("Failed to find user home directory, abort!");
            // Doesn't feel right to exit directly with a exit code of 1
            // Lmk if there is a better way of doing this
            exit(1);
        }
    })()) {
        Ok(_) => (),
        Err(err) => {
            match err {
                err => {
                    if let ReadlineError::Io(io_err) = err {
                        if io_err.kind() == ErrorKind::NotFound {
                            std::fs::write(
                                // We know for sure that the home dir already exists
                                home::home_dir().unwrap().join(".lune_history"),
                                String::new(),
                            )?;
                        }
                    }
                }
            }

            eprintln!("WARN: Failed to load REPL history")
        }
    };

    let mut interrupt_counter = 0u32;

    loop {
        let mut source_code = String::new();

        match repl.readline("> ") {
            Ok(code) => {
                source_code = code.clone();
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
                    break;
                }
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("REPL ERROR: {}", err.to_string());

                // This isn't a good way to exit imo, once again
                exit(1);
            }
        };

        let eval_result = lune_instance.run("REPL", source_code).await;

        match eval_result {
            Ok(_) => (),
            Err(err) => eprintln!("{}", err),
        };
    }

    Ok(ExitCode::SUCCESS)
}
