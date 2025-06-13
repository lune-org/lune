use std::{io::ErrorKind, process::ExitCode};

use anyhow::{Context, Result};
use async_fs as fs;
use clap::Parser;
use directories::UserDirs;
use thiserror::Error;

use serde_json::Value as JsonValue;

const LUAURC_PATH: &str = ".luaurc";

/// Set up type definitions for your editor
#[derive(Debug, Clone, Parser)]
pub struct SetupCommand {}

impl SetupCommand {
    pub async fn run(self) -> Result<ExitCode> {
        generate_typedef_files_from_definitions()
            .await
            .expect("Failed to generate typedef files");

        let mut luaurc = read_or_create_luaurc().await?;

        add_values_to_luaurc(&mut luaurc);

        write_luaurc(luaurc).await?;

        println!(
            "Type definitions for Lune v{} have been set up successfully.\
            \nYou may need to restart your editor for the changes to take effect.",
            lune_version()
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

async fn read_or_create_luaurc() -> Result<JsonValue, SetupError> {
    match fs::read(LUAURC_PATH).await {
        Err(e) if e.kind() == ErrorKind::NotFound => match fs::write(LUAURC_PATH, "{}").await {
            Err(_) => Err(SetupError::Write),
            Ok(()) => Ok(JsonValue::Object(serde_json::Map::new())),
        },
        Err(_) => Err(SetupError::Read),
        Ok(contents) => match serde_json::from_slice(&contents) {
            Err(_) => Err(SetupError::Deserialize),
            Ok(json) => Ok(json),
        },
    }
}

async fn write_luaurc(luaurc: JsonValue) -> Result<(), SetupError> {
    match serde_json::to_vec_pretty(&luaurc) {
        Err(_) => Err(SetupError::Serialize),
        Ok(mut json) => {
            json.push(b'\n');
            match fs::write(LUAURC_PATH, json).await {
                Err(_) => Err(SetupError::Write),
                Ok(()) => Ok(()),
            }
        }
    }
}

fn add_values_to_luaurc(luaurc: &mut JsonValue) {
    if let JsonValue::Object(luaurc) = luaurc {
        let field = String::from("aliases");
        let alias = String::from("lune");
        let dir = JsonValue::String(format!("~/.lune/.typedefs/{}/", lune_version()));

        if let Some(JsonValue::Object(aliases)) = luaurc.get_mut(&field) {
            if aliases.contains_key(&alias) {
                if aliases.get(&alias).unwrap() != &dir {
                    aliases.insert(alias, dir);
                }
            } else {
                aliases.insert(alias, dir);
            }
        } else {
            let mut map = serde_json::Map::new();
            map.insert(alias, dir);
            luaurc.insert(field, JsonValue::Object(map));
        }
    }
}

async fn generate_typedef_files_from_definitions() -> Result<String> {
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
    for builtin in lune_std::LuneStandardLibrary::ALL {
        let name = builtin.name().to_lowercase();
        let path = cache_dir.join(&name).with_extension("luau");
        files_to_write.push((name, path, builtin.typedefs()));
    }

    // Write all dirs and files
    for dir in dirs_to_write {
        fs::create_dir_all(dir).await?;
    }
    for (_name, path, contents) in files_to_write {
        fs::write(path, contents).await?;
    }
    Ok(version_string.to_string())
}
