use std::{
    collections::HashMap,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tokio::fs;

use super::paths::make_absolute_and_clean;

const LUAURC_FILE: &str = ".luaurc";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LuauLanguageMode {
    NoCheck,
    NonStrict,
    Strict,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LuauRcConfig {
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

#[derive(Debug, Clone)]
pub struct LuauRc {
    dir: PathBuf,
    config: LuauRcConfig,
}

impl LuauRc {
    pub async fn read(dir: impl AsRef<Path>) -> Option<Self> {
        let dir = make_absolute_and_clean(dir);
        let path = dir.join(LUAURC_FILE);
        let bytes = fs::read(&path).await.ok()?;
        let config = serde_json::from_slice(&bytes).ok()?;
        Some(Self { dir, config })
    }

    pub async fn read_recursive(
        dir: impl AsRef<Path>,
        mut predicate: impl FnMut(&Self) -> bool,
    ) -> Option<Self> {
        let mut current = make_absolute_and_clean(dir);
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

    pub fn validate(&self) -> Result<(), String> {
        if let Some(aliases) = &self.config.aliases {
            for alias in aliases.keys() {
                if !is_valid_alias_key(alias) {
                    return Err(format!("invalid alias key: {}", alias));
                }
            }
        }
        Ok(())
    }

    pub fn aliases(&self) -> HashMap<String, String> {
        self.config.aliases.clone().unwrap_or_default()
    }

    pub fn find_alias(&self, name: &str) -> Option<PathBuf> {
        self.config.aliases.as_ref().and_then(|aliases| {
            aliases.iter().find_map(|(alias, path)| {
                if alias
                    .trim_end_matches(MAIN_SEPARATOR)
                    .eq_ignore_ascii_case(name)
                    && is_valid_alias_key(alias)
                {
                    Some(self.dir.join(path).clean())
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
