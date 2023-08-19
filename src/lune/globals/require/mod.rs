use mlua::prelude::*;

use crate::lune::scheduler::LuaSchedulerExt;

mod context;
use context::RequireContext;

mod absolute;
mod alias;
mod builtin;
mod relative;

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.set_app_data(RequireContext::new());
    lua.create_async_function(|lua, path: LuaString| async move {
        let path = path
            .to_str()
            .into_lua_err()
            .context("Failed to parse require path as string")?
            .to_string();

        let context = lua
            .app_data_ref()
            .expect("Failed to get RequireContext from app data");

        let res = if let Some(builtin_name) = path
            .strip_prefix("@lune/")
            .map(|name| name.to_ascii_lowercase())
        {
            builtin::require(lua, &context, &builtin_name).await
        } else if let Some(aliased_path) = path.strip_prefix('@') {
            let (alias, name) = aliased_path.split_once('/').ok_or(LuaError::runtime(
                "Require with custom alias must contain '/' delimiter",
            ))?;
            alias::require(lua, &context, alias, name).await
        } else if context.use_absolute_paths() {
            absolute::require(lua, &context, &path).await
        } else {
            relative::require(lua, &context, &path).await
        };

        res.clone()
    })
}
