use mlua::prelude::*;

use lune_utils::get_version_string;

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    let s = get_version_string().to_string();
    lua.create_string(s)?.into_lua(lua)
}
