use mlua::prelude::*;

use crate::roblox::datatypes::extension::RobloxUserdataTypenameExt;

const REGISTRY_KEY: &str = "NetClient";

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    let original = lua.globals().get::<_, LuaFunction>("typeof")?;
    #[cfg(feature = "roblox")]
    {
        lua.set_named_registry_value(REGISTRY_KEY, original)
            .expect("Failed to store typeof function in registry");
        lua.create_function(|lua, value: LuaValue| {
            if let LuaValue::UserData(u) = &value {
                if let Some(type_name) = u.roblox_type_name() {
                    return lua.create_string(type_name);
                }
            }
            let original_fn: LuaFunction = lua
                .named_registry_value(REGISTRY_KEY)
                .expect("Missing typeof function in registry");
            original_fn.call(value)
        })
    }
    #[cfg(not(feature = "roblox"))]
    original
}
