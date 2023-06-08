use std::{
    borrow::BorrowMut,
    collections::HashMap,
    fmt::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use serde_json::Value as JsonValue;

use include_dir::{include_dir, Dir};
use lune::Lune;
use tokio::{
    fs::{self, read as read_to_vec},
    io::{stdin, AsyncReadExt},
};

use crate::{
    gen::{generate_gitbook_dir_from_definitions, generate_typedef_files_from_definitions},
    utils::{
        files::{discover_script_file_path_including_lune_dirs, strip_shebang},
        listing::{find_lune_scripts, sort_lune_scripts, write_lune_scripts_list},
    },
};

pub(crate) static TYPEDEFS_DIR: Dir<'_> = include_dir!("docs/typedefs");

/// A Luau script runner
#[derive(Parser, Debug, Default, Clone)]
#[command(version, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Script name or full path to the file to run
    script_path: Option<String>,
    /// Arguments to pass to the script, stored in process.args
    script_args: Vec<String>,
    /// List scripts found inside of a nearby `lune` directory
    #[clap(long, short = 'l')]
    list: bool,
    /// Set up type definitions and settings for development
    #[clap(long)]
    setup: bool,
    /// Generate a Luau type definitions file in the current dir
    #[clap(long, hide = true)]
    generate_luau_types: bool,
    /// Generate a Selene type definitions file in the current dir
    #[clap(long, hide = true)]
    generate_selene_types: bool,
    /// Generate a Lune documentation file for Luau LSP
    #[clap(long, hide = true)]
    generate_docs_file: bool,
    /// Generate the full Lune gitbook directory
    #[clap(long, hide = true)]
    generate_gitbook_dir: bool,
}

#[allow(dead_code)]
impl Cli {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path<S>(mut self, path: S) -> Self
    where
        S: Into<String>,
    {
        self.script_path = Some(path.into());
        self
    }

    pub fn with_args<A>(mut self, args: A) -> Self
    where
        A: Into<Vec<String>>,
    {
        self.script_args = args.into();
        self
    }

    pub fn setup(mut self) -> Self {
        self.setup = true;
        self
    }

    pub fn list(mut self) -> Self {
        self.list = true;
        self
    }

    #[allow(clippy::too_many_lines)]
    pub async fn run(self) -> Result<ExitCode> {
        // List files in `lune` and `.lune` directories, if wanted
        // This will also exit early and not run anything else
        if self.list {
            let sorted_relative = match find_lune_scripts(false).await {
                Ok(scripts) => sort_lune_scripts(scripts),
                Err(e) => {
                    eprintln!("{e}");
                    return Ok(ExitCode::FAILURE);
                }
            };
            let sorted_home_dir = match find_lune_scripts(true).await {
                Ok(scripts) => sort_lune_scripts(scripts),
                Err(e) => {
                    eprintln!("{e}");
                    return Ok(ExitCode::FAILURE);
                }
            };

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

            return Ok(ExitCode::SUCCESS);
        }
        // Generate (save) definition files, if wanted
        let generate_file_requested = self.setup
            || self.generate_luau_types
            || self.generate_selene_types
            || self.generate_docs_file
            || self.generate_gitbook_dir;
        if generate_file_requested {
            if self.generate_gitbook_dir {
                generate_gitbook_dir_from_definitions(&TYPEDEFS_DIR).await?;
            }
            if (self.generate_luau_types || self.generate_selene_types || self.generate_docs_file)
                && !self.setup
            {
                eprintln!(
                    "\
					Typedef & docs generation files have been superseded by the --setup command.\
					Run lune --setup in your terminal to configure typedef files.
					"
                );
                return Ok(ExitCode::FAILURE);
            }
            if self.setup {
                let generated_paths =
                    generate_typedef_files_from_definitions(&TYPEDEFS_DIR).await?;
                let settings_json_path = PathBuf::from(".vscode/settings.json");
                let message = match fs::metadata(&settings_json_path).await {
                    Ok(meta) if meta.is_file() => {
                        if try_add_generated_typedefs_vscode(&settings_json_path, &generated_paths).await.is_err() {
							"These files can be added to your LSP settings for autocomplete and documentation."
						} else {
							"These files have now been added to your workspace LSP settings for Visual Studio Code."
						}
                    }
                    _ => "These files can be added to your LSP settings for autocomplete and documentation.",
                };
                // HACK: We should probably just be serializing this hashmap to print it out, but
                // that does not guarantee sorting and the sorted version is much easier to read
                let mut sorted_names = generated_paths
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                sorted_names.sort_unstable();
                println!(
                    "Typedefs have been generated in the following locations:\n{{\n{}\n}}\n{message}",
                    sorted_names
                        .iter()
                        .map(|name| {
                            let path = generated_paths.get(name).unwrap();
                            format!(
                                "    \"@lune/{}\": \"{}\",",
                                name,
                                path.canonicalize().unwrap().display()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .strip_suffix(',')
                        .unwrap()
                );
            }
        }
        if self.script_path.is_none() {
            // Only generating typedefs without running a script is completely
            // fine, and we should just exit the program normally afterwards
            if generate_file_requested {
                return Ok(ExitCode::SUCCESS);
            }
            // HACK: We know that we didn't get any arguments here but since
            // script_path is optional clap will not error on its own, to fix
            // we will duplicate the cli command and make arguments required,
            // which will then fail and print out the normal help message
            let cmd = Cli::command();
            cmd.arg_required_else_help(true).get_matches();
        }
        // Figure out if we should read from stdin or from a file,
        // reading from stdin is marked by passing a single "-"
        // (dash) as the script name to run to the cli
        let script_path = self.script_path.unwrap();
        let (script_display_name, script_contents) = if script_path == "-" {
            let mut stdin_contents = Vec::new();
            stdin()
                .read_to_end(&mut stdin_contents)
                .await
                .context("Failed to read script contents from stdin")?;
            ("stdin".to_string(), stdin_contents)
        } else {
            let file_path = discover_script_file_path_including_lune_dirs(&script_path)?;
            let file_contents = read_to_vec(&file_path).await?;
            // NOTE: We skip the extension here to remove it from stack traces
            let file_display_name = file_path.with_extension("").display().to_string();
            (file_display_name, file_contents)
        };
        // Create a new lune object with all globals & run the script
        let result = Lune::new()
            .with_args(self.script_args)
            .run(&script_display_name, strip_shebang(script_contents))
            .await;
        Ok(match result {
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
            Ok(code) => code,
        })
    }
}

async fn try_add_generated_typedefs_vscode(
    settings_json_path: &Path,
    generated_paths: &HashMap<String, PathBuf>,
) -> Result<()> {
    // FUTURE: Use a jsonc or json5 to read this file instead since it may contain comments and fail
    let settings_json_contents = fs::read(settings_json_path).await?;
    let mut settings_changed: bool = false;
    let mut settings_json: JsonValue = serde_json::from_slice(&settings_json_contents)?;
    if let JsonValue::Object(settings) = settings_json.borrow_mut() {
        if let Some(JsonValue::Object(aliases)) = settings.get_mut("luau-lsp.require.fileAliases") {
            for (name, path) in generated_paths {
                settings_changed = true;
                aliases.insert(
                    format!("@lune/{name}"),
                    JsonValue::String(path.canonicalize().unwrap().to_string_lossy().to_string()),
                );
            }
        }
    }
    if settings_changed {
        let settings_json_new = serde_json::to_vec_pretty(&settings_json)?;
        fs::write(settings_json_path, settings_json_new).await?;
    }
    Ok(())
}
