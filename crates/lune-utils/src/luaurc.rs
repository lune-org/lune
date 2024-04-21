use std::{
    collections::HashMap,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::fs::read;

use crate::path::{clean_path, clean_path_and_make_absolute};

const LUAURC_FILE: &str = ".luaurc";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LuauLanguageMode {
    NoCheck,
    NonStrict,
    Strict,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LuauRcConfig {
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
    paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<HashMap<String, String>>,
}

/**
    A deserialized `.luaurc` file.

    Contains utility methods for validating and searching for aliases.
*/
#[derive(Debug, Clone)]
pub struct LuauRc {
    dir: Arc<Path>,
    config: LuauRcConfig,
}

impl LuauRc {
    /**
        Reads a `.luaurc` file from the given directory.

        If the file does not exist, or if it is invalid, this function returns `None`.
    */
    pub async fn read(dir: impl AsRef<Path>) -> Option<Self> {
        let dir = clean_path_and_make_absolute(dir);
        let path = dir.join(LUAURC_FILE);
        let bytes = read(&path).await.ok()?;
        let config = serde_json::from_slice(&bytes).ok()?;
        Some(Self {
            dir: dir.into(),
            config,
        })
    }

    /**
        Reads a `.luaurc` file from the given directory, and then recursively searches
        for a `.luaurc` file in the parent directories if a predicate is not satisfied.

        If no `.luaurc` file exists, or if they are invalid, this function returns `None`.
    */
    pub async fn read_recursive(
        dir: impl AsRef<Path>,
        mut predicate: impl FnMut(&Self) -> bool,
    ) -> Option<Self> {
        let mut current = clean_path_and_make_absolute(dir);
        loop {
            if let Some(rc) = Self::read(&current).await {
                if predicate(&rc) {
                    return Some(rc);
                }
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                return None;
            }
        }
    }

    /**
        Validates that the `.luaurc` file is correct.

        This primarily validates aliases since they are not
        validated during creation of the [`LuauRc`] struct.

        # Errors

        If an alias key is invalid.
    */
    pub fn validate(&self) -> Result<(), String> {
        if let Some(aliases) = &self.config.aliases {
            for alias in aliases.keys() {
                if !is_valid_alias_key(alias) {
                    return Err(format!("invalid alias key: {alias}"));
                }
            }
        }
        Ok(())
    }

    /**
        Gets a copy of all aliases in the `.luaurc` file.

        Will return an empty map if there are no aliases.
    */
    #[must_use]
    pub fn aliases(&self) -> HashMap<String, String> {
        self.config.aliases.clone().unwrap_or_default()
    }

    /**
        Finds an alias in the `.luaurc` file by name.

        If the alias does not exist, this function returns `None`.
    */
    #[must_use]
    pub fn find_alias(&self, name: &str) -> Option<PathBuf> {
        self.config.aliases.as_ref().and_then(|aliases| {
            aliases.iter().find_map(|(alias, path)| {
                if alias
                    .trim_end_matches(MAIN_SEPARATOR)
                    .eq_ignore_ascii_case(name)
                    && is_valid_alias_key(alias)
                {
                    Some(clean_path(self.dir.join(path)))
                } else {
                    None
                }
            })
        })
    }
}

fn is_valid_alias_key(alias: impl AsRef<str>) -> bool {
    let alias = alias.as_ref();
    if alias.is_empty()
        || alias.starts_with('.')
        || alias.starts_with("..")
        || alias.chars().any(|c| c == MAIN_SEPARATOR)
    {
        false // Paths are not valid alias keys
    } else {
        alias.chars().all(is_valid_alias_char)
    }
}

fn is_valid_alias_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
}
