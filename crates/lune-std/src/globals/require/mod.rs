use crate::{
    luaurc::{path_to_alias, Luaurc},
    path::get_parent_path,
    LuneStandardLibrary,
};
use mlua::prelude::*;
use path::resolve_path;
use std::path::PathBuf;
use thiserror::Error;

pub mod context;
mod path;

#[derive(Error, Debug)]
pub enum RequireError {
    #[error("failed to find RequireContextData in the app data container, make sure to call RequireContext::init first")]
    RequireContextNotFound,
    #[error("make sure to call RequireContext::init")]
    RequireContextInitCalledTwice,
    #[error("Can not require '{0}' as it does not exist")]
    InvalidRequire(String),
    #[error("Alias '{0}' does not point to a built-in standard library")]
    InvalidStdAlias(String),
    #[error("Library '{0}' does not point to a member of '{1}' standard libraries")]
    StdMemberNotFound(String, String),

    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TryLockError: {0}")]
    TryLockError(#[from] tokio::sync::TryLockError),
    #[error("BroadcastRecvError: {0}")]
    BroadcastRecvError(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("LuaError: {0}")]
    LuaError(#[from] mlua::Error),
}

pub async fn lua_require(lua: &Lua, path: String) -> LuaResult<LuaMultiValue> {
    let require_path_rel = PathBuf::from(path);
    let require_alias = path_to_alias(&require_path_rel).into_lua_err()?;

    if let Some(require_alias) = require_alias {
        if context::RequireContext::std_exists(lua, &require_alias.alias).into_lua_err()? {
            context::RequireContext::require_std(lua, require_alias).into_lua_err()
        } else {
            let require_path_abs = resolve_path(
                &Luaurc::resolve_path(lua, &require_alias)
                    .await
                    .into_lua_err()?,
            )
            .await?;

            context::RequireContext::require(lua, require_path_rel, require_path_abs)
                .await
                .into_lua_err()
        }
    } else {
        let parent_path = get_parent_path(lua)?;
        let require_path_abs = resolve_path(&parent_path.join(&require_path_rel))
            .await
            .map_err(|_| {
                LuaError::runtime(format!(
                    "Can not require '{}' as it does not exist",
                    require_path_rel.to_string_lossy(),
                ))
            })?;

        context::RequireContext::require(lua, require_path_rel, require_path_abs)
            .await
            .into_lua_err()
    }
}

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    let f = lua.create_async_function(lua_require).into_lua_err()?;

    context::RequireContext::init(lua).into_lua_err()?;

    for std in LuneStandardLibrary::ALL {
        context::RequireContext::inject_std(lua, "lune", *std).into_lua_err()?;
    }

    f.into_lua(lua)
}
