use crate::path::get_parent_path;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct RequireAlias {
    pub alias: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LuauLanguageMode {
    NoCheck,
    NonStrict,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Luaurc {
    #[serde(skip_serializing_if = "Option::is_none")]
    language_mode: Option<LuauLanguageMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lint: Option<HashMap<String, JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lint_errors: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_errors: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    globals: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<HashMap<String, PathBuf>>,
}

#[derive(Debug, Error)]
pub enum LuaurcError {
    #[error("Require with alias doesn't contain '/'")]
    UsedAliasWithoutSlash,
    #[error("Failed to convert string to path")]
    FailedStringToPathConversion,
    #[error("Failed to find a path for alias '{0}' in .luaurc files")]
    FailedToFindAlias(String),
    #[error("Failed to parse {0}\nParserError: {1}")]
    FilaedToParse(PathBuf, serde_json::Error),

    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("LuaError: {0}")]
    LuaError(#[from] mlua::Error),
}

impl RequireAlias {
    /// Parses path into `RequireAlias` struct
    ///
    /// ### Examples
    ///
    /// `@lune/task` becomes `Some({ alias: "lune", path: "task" })`
    ///
    /// `../path/script` becomes `None`
    pub fn from_path(path: &Path) -> Result<Option<Self>, LuaurcError> {
        if let Some(aliased_path) = path
            .to_str()
            .ok_or(LuaurcError::FailedStringToPathConversion)?
            .strip_prefix('@')
        {
            let (alias, path) = aliased_path
                .split_once('/')
                .ok_or(LuaurcError::UsedAliasWithoutSlash)?;

            Ok(Some(RequireAlias {
                alias: alias.to_string(),
                path: path.to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}

async fn parse_luaurc(_: &mlua::Lua, path: &PathBuf) -> Result<Option<Luaurc>, LuaurcError> {
    if fs::try_exists(path).await? {
        let content = fs::read(path).await?;
        serde_json::from_slice(&content)
            .map(Some)
            .map_err(|err| LuaurcError::FilaedToParse(path.clone(), err))
    } else {
        Ok(None)
    }
}

impl Luaurc {
    /// Searches for .luaurc recursively
    /// until an alias for the provided `RequireAlias` is found
    pub async fn resolve_path<'lua>(
        lua: &'lua mlua::Lua,
        alias: &'lua RequireAlias,
    ) -> Result<PathBuf, LuaurcError> {
        let cwd = current_dir()?;
        let parent = cwd.join(get_parent_path(lua)?);
        let ancestors = parent.ancestors();

        for path in ancestors {
            if path.starts_with(&cwd) {
                if let Some(luaurc) = parse_luaurc(lua, &path.join(".luaurc")).await? {
                    if let Some(aliases) = luaurc.aliases {
                        if let Some(alias_path) = aliases.get(&alias.alias) {
                            let resolved = path.join(alias_path.join(&alias.path));

                            return Ok(resolved);
                        }
                    }
                }
            } else {
                break;
            }
        }

        Err(LuaurcError::FailedToFindAlias(alias.alias.to_string()))
    }
}
