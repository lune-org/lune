use std::{
    borrow::BorrowMut, collections::HashMap, env::current_dir, io::ErrorKind, path::PathBuf,
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::Parser;
use directories::UserDirs;
use futures_util::future::try_join_all;
use include_dir::{include_dir, Dir};
use thiserror::Error;
use tokio::fs;

// TODO: Use a library that supports json with comments since VSCode settings may contain comments
use serde_json::Value as JsonValue;

pub(crate) static TYPEDEFS_DIR: Dir<'_> = include_dir!("types");

pub(crate) static SETTING_NAME_MODE: &str = "luau-lsp.require.mode";
pub(crate) static SETTING_NAME_ALIASES: &str = "luau-lsp.require.directoryAliases";

/// Set up type definitions for your editor
#[derive(Debug, Clone, Parser)]
pub struct SetupCommand {}

impl SetupCommand {
    pub async fn run(self) -> Result<ExitCode> {
        generate_typedef_files_from_definitions(&TYPEDEFS_DIR)
            .await
            .expect("Failed to generate typedef files");

        // TODO: Let the user interactively choose what editor to set up
        let res = async {
            let settings = read_or_create_vscode_settings_json().await?;
            let modified = add_values_to_vscode_settings_json(settings);
            write_vscode_settings_json(modified).await?;
            Ok::<_, SetupError>(())
        }
        .await;

        let message = match res {
            Ok(()) => "These settings have been added to your workspace for Visual Studio Code:",
            Err(_) => "To finish setting up your editor, add these settings to your workspace:",
        };

        let version_string = lune_version();
        println!(
            "Lune has now been set up and editor type definitions have been generated.\
            \n{message}\
            \n\
            \n\"{SETTING_NAME_MODE}\": \"relativeToFile\",\
            \n\"{SETTING_NAME_ALIASES}\": {{\
            \n    \"@lune/\": \"~/.lune/.typedefs/{version_string}/\"\
            \n}}",
        );

        Ok(ExitCode::SUCCESS)
    }
}

#[derive(Debug, Clone, Copy, Error)]
enum SetupError {
    #[error("Failed to read settings")]
    Read,
    #[error("Failed to write settings")]
    Write,
    #[error("Failed to parse settings")]
    Deserialize,
    #[error("Failed to create settings")]
    Serialize,
}

fn lune_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn vscode_path() -> PathBuf {
    current_dir()
        .expect("No current dir")
        .join(".vscode")
        .join("settings.json")
}

async fn read_or_create_vscode_settings_json() -> Result<JsonValue, SetupError> {
    let path_file = vscode_path();
    let mut path_dir = path_file.clone();
    path_dir.pop();
    match fs::read(&path_file).await {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // TODO: Make sure that VSCode is actually installed, or
            // let the user choose their editor for interactive setup
            match fs::create_dir_all(path_dir).await {
                Err(_) => Err(SetupError::Write),
                Ok(()) => match fs::write(path_file, "{}").await {
                    Err(_) => Err(SetupError::Write),
                    Ok(()) => Ok(JsonValue::Object(serde_json::Map::new())),
                },
            }
        }
        Err(_) => Err(SetupError::Read),
        Ok(contents) => match serde_json::from_slice(&contents) {
            Err(_) => Err(SetupError::Deserialize),
            Ok(json) => Ok(json),
        },
    }
}

async fn write_vscode_settings_json(value: JsonValue) -> Result<(), SetupError> {
    match serde_json::to_vec_pretty(&value) {
        Err(_) => Err(SetupError::Serialize),
        Ok(json) => match fs::write(vscode_path(), json).await {
            Err(_) => Err(SetupError::Write),
            Ok(()) => Ok(()),
        },
    }
}

fn add_values_to_vscode_settings_json(value: JsonValue) -> JsonValue {
    let mut settings_json = value;
    if let JsonValue::Object(settings) = settings_json.borrow_mut() {
        // Set require mode
        let mode_val = "relativeToFile".to_string();
        settings.insert(SETTING_NAME_MODE.to_string(), JsonValue::String(mode_val));
        // Set require alias to our typedefs
        let aliases_key = "@lune/".to_string();
        let aliases_val = format!("~/.lune/.typedefs/{}/", lune_version());
        if let Some(JsonValue::Object(aliases)) = settings.get_mut(SETTING_NAME_ALIASES) {
            if aliases.contains_key(&aliases_key) {
                if aliases.get(&aliases_key).unwrap() != &JsonValue::String(aliases_val.to_string())
                {
                    aliases.insert(aliases_key, JsonValue::String(aliases_val));
                }
            } else {
                aliases.insert(aliases_key, JsonValue::String(aliases_val));
            }
        } else {
            let mut map = serde_json::Map::new();
            map.insert(aliases_key, JsonValue::String(aliases_val));
            settings.insert(SETTING_NAME_ALIASES.to_string(), JsonValue::Object(map));
        }
    }
    settings_json
}

async fn generate_typedef_files_from_definitions(dir: &Dir<'_>) -> Result<String> {
    let contents = read_typedefs_dir_contents(dir);
    write_typedef_files(contents).await
}

fn read_typedefs_dir_contents(dir: &Dir<'_>) -> HashMap<String, Vec<u8>> {
    let mut definitions = HashMap::new();

    for entry in dir.find("*.luau").unwrap() {
        let entry_file = entry.as_file().unwrap();
        let entry_name = entry_file.path().file_name().unwrap().to_string_lossy();

        let typedef_name = entry_name.trim_end_matches(".luau");
        let typedef_contents = entry_file.contents().to_vec();

        definitions.insert(typedef_name.to_string(), typedef_contents);
    }

    definitions
}

async fn write_typedef_files(typedef_files: HashMap<String, Vec<u8>>) -> Result<String> {
    let version_string = env!("CARGO_PKG_VERSION");
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the typedefs dir in the users cache dir
    let cache_dir = UserDirs::new()
        .context("Failed to find user home directory")?
        .home_dir()
        .join(".lune")
        .join(".typedefs")
        .join(version_string);
    dirs_to_write.push(cache_dir.clone());
    // Make typedef files
    for (builtin_name, builtin_typedef) in typedef_files {
        let path = cache_dir
            .join(builtin_name.to_ascii_lowercase())
            .with_extension("luau");
        files_to_write.push((builtin_name.to_lowercase(), path, builtin_typedef));
    }
    // Write all dirs and files only when we know generation was successful
    let futs_dirs = dirs_to_write
        .drain(..)
        .map(fs::create_dir_all)
        .collect::<Vec<_>>();
    let futs_files = files_to_write
        .iter()
        .map(|(_, path, contents)| fs::write(path, contents))
        .collect::<Vec<_>>();
    try_join_all(futs_dirs).await?;
    try_join_all(futs_files).await?;
    Ok(version_string.to_string())
}
