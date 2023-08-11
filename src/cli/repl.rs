use std::{
    env,
    fmt::Write,
    io::ErrorKind,
    path::PathBuf,
    process::{exit, ExitCode},
};

use anyhow::Error;
use clap::Command;
use regex::Regex;
use lune::lua::stdio::formatting::{pretty_format_luau_error, pretty_format_value};
use lune::Lune;
use mlua::ExternalError;
use once_cell::sync::Lazy;
use rustyline::{error::ReadlineError, history::FileHistory, DefaultEditor, Editor};

fn env_var_bool(value: String) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        &_ => None,
    }
}

// Isn't dependency injection plain awesome?!
pub async fn show_interface(cmd: Command) -> Result<ExitCode, Error> {
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

    let colorize: Lazy<bool> = Lazy::new(|| {
        let no_color = env::var("NO_COLOR").unwrap_or_else(|_| "false".to_string());

        if no_color.is_empty() {
            true
        } else {
            !env_var_bool(no_color).unwrap_or_else(|| false)
        }
    });
    
    const VARIABLE_DECLARATION_PAT: &str = r#"(local)?\s*(.*)\s*(=)\s*(.*)\s*"#;

    // HACK: Prepend this "context" to the source code provided,
    // so that the variable is preserved even the following steps
    let mut source_code_context: Option<String> = None;

    loop {
        let mut source_code = String::new();

        match repl.readline("> ") {
            Ok(code) => {
                if let Some(ref ctx) = source_code_context {
                    // If something breaks, blame this
                    source_code = format!("{}    {}", ctx, code);
                } else {
                    source_code.push_str(code.as_str());
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
            Ok(_) => {
                if Regex::new(VARIABLE_DECLARATION_PAT)?.is_match(&source_code)
                    && !&source_code.contains("\n")
                {
                    let declaration = source_code.split("=").collect::<Vec<&str>>()[1]
                        .trim()
                        .replace("\"", "");

                    let mut formatted_output = String::new();
                    pretty_format_value(
                        &mut formatted_output,
                        &mlua::IntoLua::into_lua(declaration, &mlua::Lua::new())?,
                        1,
                    )?;

                    source_code_context = (|| -> Result<Option<String>, Error> {
                        let mut ctx = String::new();
                        write!(&mut ctx, "{}\n", &source_code)?;

                        Ok(Some(ctx))
                    })()?;

                    println!("{}", formatted_output);
                }
            }

            Err(err) => {
                eprintln!(
                    "{}",
                    pretty_format_luau_error(&err.into_lua_err(), (&*colorize).to_owned())
                )
            }
        };
    }

    Ok(ExitCode::SUCCESS)
}

fn save_repl_activity(mut repl: Editor<(), FileHistory>) -> Result<(), Error> {
    // Once again, we know that the specified home directory
    // and history file already exist
    repl.save_history(&home::home_dir().unwrap().join(".lune_history"))?;

    Ok(())
}
