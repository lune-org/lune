use std::{env, fmt::Write as _, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};
use clap::Parser;

use lune::Lune;
use tokio::{
    fs::read as read_to_vec,
    io::{stdin, AsyncReadExt},
};

pub(crate) mod build;
pub(crate) mod gen;
pub(crate) mod repl;
pub(crate) mod setup;
pub(crate) mod utils;

use setup::run_setup;
use utils::{
    files::{discover_script_path_including_lune_dirs, strip_shebang},
    listing::{find_lune_scripts, sort_lune_scripts, write_lune_scripts_list},
};

use self::build::build_standalone;

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
    #[clap(long, hide = true)]
    build: bool,
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
        // Signature which is only present in standalone lune binaries
        let signature: Vec<u8> = vec![0x4f, 0x3e, 0xf8, 0x41, 0xc3, 0x3a, 0x52, 0x16];
        // Read the current lune binary to memory
        let bin = read_to_vec(env::current_exe()?).await?;

        let is_standalone = bin[bin.len() - signature.len()..bin.len()] == signature;

        // List files in `lune` and `.lune` directories, if wanted
        // This will also exit early and not run anything else
        if self.list && !is_standalone {
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

            return Ok(ExitCode::SUCCESS);
        }

        // Generate (save) definition files, if wanted
        let generate_file_requested = self.setup
            || self.generate_luau_types
            || self.generate_selene_types
            || self.generate_docs_file;
        if generate_file_requested {
            if (self.generate_luau_types || self.generate_selene_types || self.generate_docs_file)
                && !self.setup
            {
                eprintln!(
                    "\
					Typedef & docs generation commands have been superseded by the setup command.\
					Run `lune --setup` in your terminal to configure your editor and type definitions.
					"
                );
                return Ok(ExitCode::FAILURE);
            }
            if self.setup {
                run_setup().await;
            }
        }
        if self.script_path.is_none() {
            // Only generating typedefs without running a script is completely
            // fine, and we should just exit the program normally afterwards
            if generate_file_requested {
                return Ok(ExitCode::SUCCESS);
            }

            if is_standalone {
                let mut bytecode_offset = 0;
                let mut bytecode_size = 0;

                // standalone binary structure (reversed, 8 bytes per field)
                // [0] => signature
                // ----------------
                // -- META Chunk --
                // [1] => file count
                // [2] => bytecode size
                // [3] => bytecode offset
                // ----------------
                // -- MISC Chunk --
                // [4..n] => bytecode (variable size)
                // ----------------
                // NOTE: All integers are 8 byte unsigned 64 bit (u64's).

                // The rchunks will have unequally sized sections in the beginning
                // but that doesn't matter to us because we don't need anything past the
                // middle chunks where the bytecode is stored
                for (idx, chunk) in bin.rchunks(signature.len()).enumerate() {
                    if idx == 0 && chunk != signature {
                        // We don't have a standalone binary
                        break;
                    }

                    if idx == 3 {
                        bytecode_offset = u64::from_ne_bytes(chunk.try_into()?);
                    }

                    if idx == 2 {
                        bytecode_size = u64::from_ne_bytes(chunk.try_into()?);
                    }
                }

                // If we were able to retrieve the required metadata, we load
                // and execute the bytecode
                if bytecode_offset != 0 && bytecode_size != 0 {
                    // FIXME: Passing arguments does not work like it should, because the first
                    // argument provided is treated as the script path. We should probably also not
                    // allow any runner functionality within standalone binaries

                    let result = Lune::new()
                        .with_args(self.script_args.clone()) // TODO: args should also include lune reserved ones
                        .run(
                            "STANDALONE",
                            &bin[usize::try_from(bytecode_offset).unwrap()
                                ..usize::try_from(bytecode_offset + bytecode_size).unwrap()],
                        )
                        .await;

                    return Ok(match result {
                        Err(err) => {
                            eprintln!("{err}");
                            ExitCode::FAILURE
                        }
                        Ok(code) => code,
                    });
                }
            }

            // If not in a standalone context and we don't have any arguments
            // display the interactive REPL interface
            return repl::show_interface().await;
        }

        if !is_standalone {
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
                let file_path = discover_script_path_including_lune_dirs(&script_path)?;
                let file_contents = read_to_vec(&file_path).await?;
                // NOTE: We skip the extension here to remove it from stack traces
                let file_display_name = file_path.with_extension("").display().to_string();
                (file_display_name, file_contents)
            };

            if self.build {
                let output_path =
                    PathBuf::from(script_path.clone()).with_extension(env::consts::EXE_EXTENSION);
                println!(
                    "Building {script_path} to {}",
                    output_path.to_string_lossy()
                );

                return Ok(
                    match build_standalone(output_path, strip_shebang(script_contents.clone()))
                        .await
                    {
                        Ok(exitcode) => exitcode,
                        Err(err) => {
                            eprintln!("{err}");
                            ExitCode::FAILURE
                        }
                    },
                );
            }

            // Create a new lune object with all globals & run the script
            let result = Lune::new()
                .with_args(self.script_args)
                .run(&script_display_name, strip_shebang(script_contents))
                .await;
            return Ok(match result {
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
                Ok(code) => code,
            });
        }

        Ok(ExitCode::SUCCESS)
    }
}
