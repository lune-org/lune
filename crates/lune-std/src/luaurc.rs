use crate::path::get_parent_path;
use mlua::ExternalResult;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
};
use tokio::fs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RequireAlias<'a> {
    pub alias: &'a str,
    pub path: &'a str,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Luaurc {
    aliases: Option<HashMap<String, PathBuf>>,
}

/// Parses path into `RequireAlias` struct
///
/// ### Examples
///
/// `@lune/task` becomes `Some({ alias: "lune", path: "task" })`
///
/// `../path/script` becomes `None`
pub fn path_to_alias(path: &Path) -> Result<Option<RequireAlias<'_>>, mlua::Error> {
    if let Some(aliased_path) = path
        .to_str()
        .ok_or(mlua::Error::runtime("Couldn't turn path into string"))?
        .strip_prefix('@')
    {
        let (alias, path) = aliased_path.split_once('/').ok_or(mlua::Error::runtime(
            "Require with alias doesn't contain '/'",
        ))?;

        Ok(Some(RequireAlias { alias, path }))
    } else {
        Ok(None)
    }
}

async fn parse_luaurc(_: &mlua::Lua, path: &PathBuf) -> Result<Option<Luaurc>, mlua::Error> {
    if fs::try_exists(path).await? {
        let content = fs::read(path).await?;
        serde_json::from_slice(&content).map(Some).into_lua_err()
    } else {
        Ok(None)
    }
}

/// Searches for .luaurc recursively
/// until an alias for the provided `RequireAlias` is found
pub async fn resolve_require_alias<'lua>(
    lua: &'lua mlua::Lua,
    alias: &'lua RequireAlias<'lua>,
) -> Result<PathBuf, mlua::Error> {
    let cwd = current_dir()?;
    let parent = cwd.join(get_parent_path(lua)?);
    let ancestors = parent.ancestors();

    for path in ancestors {
        if path.starts_with(&cwd) {
            if let Some(luaurc) = parse_luaurc(lua, &parent.join(".luaurc")).await? {
                if let Some(aliases) = luaurc.aliases {
                    if let Some(alias_path) = aliases.get(alias.alias) {
                        let resolved = path.join(alias_path.join(alias.path));

                        return Ok(resolved);
                    }
                }
            }
        } else {
            break;
        }
    }

    Err(mlua::Error::runtime(format!(
        "Coudln't find the alias '{}' in any .luaurc file",
        alias.alias
    )))
}
