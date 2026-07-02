use mlua::prelude::*;

use crate::require::RequireResolver;

pub fn create(lua: Lua) -> LuaResult<LuaValue> {
    lua.create_require_function(RequireResolver::new())
        .map(LuaValue::Function)
}
