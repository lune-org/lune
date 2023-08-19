use mlua::prelude::*;

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_table()
}
