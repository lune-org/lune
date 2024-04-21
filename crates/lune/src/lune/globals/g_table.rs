use mlua::prelude::*;

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_table()
}
