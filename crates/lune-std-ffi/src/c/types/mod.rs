mod c_int;

use mlua::prelude::*;

// export all default c-types
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    Ok(vec![c_int::get_export(lua)?])
}
