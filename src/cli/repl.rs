use std::{
    io::ErrorKind,
    path::PathBuf,
    process::{exit, ExitCode},
};

use anyhow::Result;
use clap::Command;
use directories::UserDirs;
use lune::lua::stdio::formatting::pretty_format_luau_error;
use lune::Lune;
use mlua::ExternalError;
use rustyline::{error::ReadlineError, history::FileHistory, DefaultEditor, Editor};

// Isn't dependency injection plain awesome?!
pub async fn show_interface(cmd: Command) -> Result<ExitCode> {
    let lune_version = cmd.get_version();

    // The version is mandatory and will always exist
    println!("Lune v{}", lune_version.unwrap());

    let lune_instance = Lune::new();

    let mut repl = DefaultEditor::new()?;

    match repl.load_history(&(|| -> PathBuf {
        let dir_opt = UserDirs::new();

        if let Some(dirs) = dir_opt {
            let home_dir = dirs.home_dir();

            home_dir.join(".lune_history")
        } else {
            eprintln!("Failed to find user home directory, abort!");
            // Doesn't feel right to exit directly with a exit code of 1
            // Lmk if there is a better way of doing this
            exit(1);
        }
    })()) {
        Ok(_) => (),
        Err(err) => {
            if let ReadlineError::Io(io_err) = err {
                // If global history file does not exist, we create it
                if io_err.kind() == ErrorKind::NotFound {
                    std::fs::write(
                        // We know for sure that the home dir already exists
                        directories::UserDirs::new()
                            .unwrap()
                            .home_dir()
                            .join(".lune_history"),
                        String::new(),
                    )?;
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
                    save_repl_activity(repl)?;
                    break;
                }
            }
            Err(ReadlineError::Eof) => {
                save_repl_activity(repl)?;
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
            Ok(_) => (),

            Err(err) => {
                eprintln!("{}", pretty_format_luau_error(&err.into_lua_err(), true))
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}

fn save_repl_activity(mut repl: Editor<(), FileHistory>) -> Result<()> {
    // Once again, we know that the specified home directory
    // and history file already exist
    repl.save_history(
        &directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .join(".lune_history"),
    )?;

    Ok(())
}
