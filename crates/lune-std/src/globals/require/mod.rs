use crate::{luaurc::path_to_alias, path::get_parent_path, LuneStandardLibrary};
use mlua::prelude::*;
use path::resolve_path;
use std::path::PathBuf;

mod context;
mod path;

pub async fn lua_require(lua: &Lua, path: String) -> LuaResult<LuaMultiValue> {
    let require_path_rel = PathBuf::from(path);
    let require_alias = path_to_alias(&require_path_rel)?;

    if let Some(require_alias) = require_alias {
        if context::RequireContext::std_exists(lua, &require_alias.alias)? {
            context::RequireContext::require_std(lua, require_alias)
        } else {
            Err(LuaError::runtime(format!(
                "Tried requiring a custom alias '{}'\nbut aliases are not implemented yet.",
                require_alias.alias,
            )))
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

        context::RequireContext::require(lua, require_path_rel, require_path_abs).await
    }
}

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    let f = lua.create_async_function(lua_require)?;

    context::RequireContext::init(lua)?;

    for std in LuneStandardLibrary::ALL {
        context::RequireContext::inject_std(lua, "lune", *std)?;
    }

    f.into_lua(lua)
}
