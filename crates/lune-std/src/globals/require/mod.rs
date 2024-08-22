use crate::{luaurc::path_to_alias, path::get_parent_path, LuneStandardLibrary};
use mlua::prelude::*;
use std::path::PathBuf;

pub mod context;

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
        let require_path_abs = parent_path.join(&require_path_rel);

        Err(LuaError::runtime(format!(
            "Tried requiring '{}'\nbut requires are not implemented yet.",
            require_path_abs.to_string_lossy(),
        )))
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
