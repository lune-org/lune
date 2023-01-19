use std::{fs::read_to_string, path::PathBuf};

use clap::Parser;
use mlua::{Lua, Result};

use crate::lune::{fs::LuneFs, json::LuneJson, process::LuneProcess};

/// Lune CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the file to run
    path: String,
}

impl Cli {
    #[allow(dead_code)]
    pub fn from_path<S: AsRef<str>>(path: S) -> Self {
        Self {
            path: path.as_ref().to_owned(),
        }
    }

    fn parse_file_path(&self) -> Result<PathBuf> {
        let parsed_file_path = match &self.path {
            path if path.ends_with(".luau") || path.ends_with(".lua") => Some(PathBuf::from(path)),
            path => {
                let temp_path = PathBuf::from(path);
                if temp_path.extension().is_none() {
                    let as_luau_path = temp_path.with_extension("luau");
                    let as_lua_path = temp_path.with_extension("lua");
                    if as_luau_path.exists() {
                        Some(as_luau_path)
                    } else if as_lua_path.exists() {
                        Some(as_lua_path)
                    } else {
                        let as_luau_in_scripts_folder = PathBuf::from("scripts").join(as_luau_path);
                        let as_lua_in_scripts_folder = PathBuf::from("scripts").join(as_lua_path);
                        if as_luau_in_scripts_folder.exists() {
                            Some(as_luau_in_scripts_folder)
                        } else if as_lua_in_scripts_folder.exists() {
                            Some(as_lua_in_scripts_folder)
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };
        if let Some(file_path) = parsed_file_path {
            if file_path.exists() {
                Ok(file_path)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "File does not exist at path: '{}'",
                    self.path
                )))
            }
        } else {
            Err(mlua::Error::RuntimeError(format!(
                "Invalid file path: '{}'",
                self.path
            )))
        }
    }

    pub async fn run(self) -> Result<()> {
        // Parse and read the wanted file
        let file_path = self.parse_file_path()?;
        let file_contents = read_to_string(file_path)?;
        // Create a new lua state and add in all lune globals
        let lua = Lua::new();
        let globals = lua.globals();
        globals.set("fs", LuneFs::new())?;
        globals.set("process", LuneProcess::new())?;
        globals.set("json", LuneJson::new())?;
        lua.sandbox(true)?;
        // Run the file
        lua.load(&file_contents).exec_async().await?;
        Ok(())
    }
}
