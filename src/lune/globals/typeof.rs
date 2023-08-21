use mlua::prelude::*;

use crate::roblox::datatypes::extension::RobloxUserdataTypenameExt;

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_function(|lua, value: LuaValue| {
        #[cfg(feature = "roblox")]
        if let LuaValue::UserData(u) = &value {
            if let Some(type_name) = u.roblox_type_name() {
                return lua.create_string(type_name);
            }
        }
        lua.globals().get::<_, LuaFunction>("typeof")?.call(value)
    })
}
