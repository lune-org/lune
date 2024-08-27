mod c_char;
mod c_double;
mod c_float;
mod c_int;
mod c_long;

use mlua::prelude::*;

// export all default c-types
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    Ok(vec![
        c_char::get_export(lua)?,
        c_double::get_export(lua)?,
        c_float::get_export(lua)?,
        c_int::get_export(lua)?,
        c_long::get_export(lua)?,
    ])
}
