use std::{
    fs::read_to_string,
    path::{PathBuf, MAIN_SEPARATOR},
};

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

    pub async fn run(self) -> Result<()> {
        // Parse and read the wanted file
        let file_path = find_parse_file_path(&self.path)?;
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

fn find_luau_file_path(path: &str) -> Option<PathBuf> {
    let file_path = PathBuf::from(path);
    if let Some(ext) = file_path.extension() {
        match ext {
            e if e == "lua" || e == "luau" && file_path.exists() => Some(file_path),
            _ => None,
        }
    } else {
        let file_path_lua = PathBuf::from(path).with_extension("lua");
        if file_path_lua.exists() {
            Some(file_path_lua)
        } else {
            let file_path_luau = PathBuf::from(path).with_extension("luau");
            if file_path_luau.exists() {
                Some(file_path_luau)
            } else {
                None
            }
        }
    }
}

fn find_parse_file_path(path: &str) -> Result<PathBuf> {
    let parsed_file_path = find_luau_file_path(path)
        .or_else(|| find_luau_file_path(&format!("lune{MAIN_SEPARATOR}{path}")))
        .or_else(|| find_luau_file_path(&format!(".lune{MAIN_SEPARATOR}{path}")));
    if let Some(file_path) = parsed_file_path {
        if file_path.exists() {
            Ok(file_path)
        } else {
            Err(mlua::Error::RuntimeError(format!(
                "File does not exist at path: '{}'",
                path
            )))
        }
    } else {
        Err(mlua::Error::RuntimeError(format!(
            "Invalid file path: '{}'",
            path
        )))
    }
}
