use mlua::prelude::*;

pub fn get_table_type_metavalue(tab: &LuaTable) -> Option<String> {
    let s = tab
        .metatable()?
        .get::<LuaString>(LuaMetaMethod::Type.name())
        .ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn get_userdata_type_metavalue(tab: &LuaAnyUserData) -> Option<String> {
    let s = tab
        .metatable()
        .ok()?
        .get::<LuaString>(LuaMetaMethod::Type.name())
        .ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn call_table_tostring_metamethod(tab: &LuaTable) -> Option<String> {
    tab.metatable()?
        .get::<LuaFunction>(LuaMetaMethod::ToString.name())
        .ok()?
        .call(tab)
        .ok()
}

pub fn call_userdata_tostring_metamethod(tab: &LuaAnyUserData) -> Option<String> {
    tab.metatable()
        .ok()?
        .get::<LuaFunction>(LuaMetaMethod::ToString.name())
        .ok()?
        .call(tab)
        .ok()
}
