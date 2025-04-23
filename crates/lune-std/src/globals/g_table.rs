use mlua::prelude::*;

pub fn create(lua: Lua) -> LuaResult<LuaValue> {
    lua.create_table()?.into_lua(&lua)
}
