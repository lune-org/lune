use mlua::prelude::*;

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    let f = lua.create_function(|_, args: LuaMultiValue| {
        // TODO: Port this over from the old crate
        Ok(())
    })?;
    f.into_lua(lua)
}
