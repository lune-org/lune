use mlua::prelude::*;

pub fn get_table_type_metavalue(tab: &LuaTable) -> Option<String> {
    let meta = tab.metatable()?;
    let s = meta.get::<LuaString>(LuaMetaMethod::Type.name()).ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn get_userdata_type_metavalue(usr: &LuaAnyUserData) -> Option<String> {
    let meta = usr.metatable().ok()?;
    let s = meta.get::<LuaString>(LuaMetaMethod::Type.name()).ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn call_table_tostring_metamethod(tab: &LuaTable) -> Option<String> {
    let meta = tab.metatable()?;
    let value = meta.get(LuaMetaMethod::ToString.name()).ok()?;
    match value {
        LuaValue::String(s) => Some(s.to_string_lossy().to_string()),
        LuaValue::Function(f) => f.call(tab).ok(),
        _ => None,
    }
}

pub fn call_userdata_tostring_metamethod(usr: &LuaAnyUserData) -> Option<String> {
    let meta = usr.metatable().ok()?;
    let value = meta.get(LuaMetaMethod::ToString.name()).ok()?;
    match value {
        LuaValue::String(s) => Some(s.to_string_lossy().to_string()),
        LuaValue::Function(f) => f.call(usr).ok(),
        _ => None,
    }
}
