use std::{
    fs::read_to_string,
    path::{PathBuf, MAIN_SEPARATOR},
};

use clap::Parser;
use mlua::{Lua, MultiValue, Result, ToLua};

use crate::lune::{fs::LuneFs, json::LuneJson, process::LuneProcess};

/// Lune CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the file to run, or the name
    /// of a luau file in a lune directory
    script_path: String,
    /// Arguments to pass to the file as vararg (...)
    script_args: Vec<String>,
    /// Pass this flag to download the Selene type
    /// definitions file to the current directory
    #[clap(long)]
    download_selene_types: bool,
    /// Pass this flag to download the Luau type
    /// definitions file to the current directory
    #[clap(long)]
    download_luau_types: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            script_path: "".to_string(),
            script_args: vec![],
            download_selene_types: false,
            download_luau_types: false,
        }
    }
}

impl Cli {
    #[allow(dead_code)]
    pub fn from_path<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            script_path: path.into(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn from_path_with_args<S, A>(path: S, args: A) -> Self
    where
        S: Into<String>,
        A: Into<Vec<String>>,
    {
        Self {
            script_path: path.into(),
            script_args: args.into(),
            ..Default::default()
        }
    }

    pub async fn run(self) -> Result<()> {
        // Parse and read the wanted file
        let file_path = find_parse_file_path(&self.script_path)?;
        let file_contents = read_to_string(file_path)?;
        // Create a new lua state and add in all lune globals
        let lua = Lua::new();
        let globals = lua.globals();
        globals.set("fs", LuneFs::new())?;
        globals.set("process", LuneProcess::new())?;
        globals.set("json", LuneJson::new())?;
        lua.sandbox(true)?;
        // Load & call the file with the given args
        let lua_args = self
            .script_args
            .iter()
            .map(|value| value.to_owned().to_lua(&lua))
            .collect::<Result<Vec<_>>>()?;
        lua.load(&file_contents)
            .call_async(MultiValue::from_vec(lua_args))
            .await?;
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
