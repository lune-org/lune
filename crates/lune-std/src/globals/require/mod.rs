use crate::{
    luaurc::{Luaurc, RequireAlias},
    path::get_parent_path,
    LuneStandardLibrary,
};
use lune_utils::path::clean_path_and_make_absolute;
use mlua::prelude::*;
use path::append_extension;
use std::path::PathBuf;
use thiserror::Error;

pub mod context;
mod path;

#[derive(Error, Debug)]
pub enum RequireError {
    #[error("failed to find RequireContextData in the app data container, make sure to call RequireContext::init first")]
    RequireContextNotFound,
    #[error("RequireContext::init has been called twice on the same Lua instance")]
    RequireContextInitCalledTwice,
    #[error("Can not require '{0}' as it does not exist")]
    InvalidRequire(String),
    #[error("Alias '{0}' does not point to a built-in standard library")]
    InvalidStdAlias(String),
    #[error("Library '{0}' does not point to a member of '{1}' standard libraries")]
    StdMemberNotFound(String, String),
    #[error("Thread result returned none")]
    ThreadReturnedNone,
    #[error("Could not get '{0}' from cache")]
    CacheNotFound(String),

    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TryLockError: {0}")]
    TryLockError(#[from] tokio::sync::TryLockError),
    #[error("BroadcastRecvError: {0}")]
    BroadcastRecvError(#[from] tokio::sync::broadcast::error::RecvError),
    #[error("LuaError: {0}")]
    LuaError(#[from] mlua::Error),
}

/**
tries different extensions on the path and if all alternatives fail, we'll try to look for an init file
 */
async fn try_alternatives(lua: &Lua, require_path_abs: PathBuf) -> LuaResult<LuaMultiValue> {
    for ext in ["lua", "luau"] {
        // try the path with ext
        let ext_path = append_extension(&require_path_abs, ext);

        match context::RequireContext::require(lua, ext_path).await {
            Ok(res) => return Ok(res),
            Err(err) => {
                if !matches!(err, RequireError::IOError(_)) {
                    return Err(err).into_lua_err();
                };
            }
        };
    }

    for ext in ["lua", "luau"] {
        // append init to path and try it with ext
        let ext_path = append_extension(require_path_abs.join("init"), ext);

        match context::RequireContext::require(lua, ext_path).await {
            Ok(res) => return Ok(res),
            Err(err) => {
                if !matches!(err, RequireError::IOError(_)) {
                    return Err(err).into_lua_err();
                };
            }
        };
    }

    Err(RequireError::InvalidRequire(
        require_path_abs.to_string_lossy().to_string(),
    ))
    .into_lua_err()
}

async fn lua_require(lua: &Lua, path: String) -> LuaResult<LuaMultiValue> {
    let require_path_rel = PathBuf::from(path);
    let require_alias = RequireAlias::from_path(&require_path_rel).into_lua_err()?;

    if let Some(require_alias) = require_alias {
        if context::RequireContext::std_exists(lua, &require_alias.alias).into_lua_err()? {
            context::RequireContext::require_std(lua, require_alias).into_lua_err()
        } else {
            let require_path_abs = clean_path_and_make_absolute(
                Luaurc::resolve_path(lua, &require_alias)
                    .await
                    .into_lua_err()?,
            );

            try_alternatives(lua, require_path_abs).await
        }
    } else {
        let parent_path = get_parent_path(lua)?;
        let require_path_abs = clean_path_and_make_absolute(parent_path.join(&require_path_rel));

        try_alternatives(lua, require_path_abs).await
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
