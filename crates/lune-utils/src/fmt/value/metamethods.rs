use mlua::prelude::*;

pub fn get_table_type_metavalue<'a>(tab: &'a LuaTable<'a>) -> Option<String> {
    let s = tab
        .get_metatable()?
        .get::<_, LuaString>(LuaMetaMethod::Type.name())
        .ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn get_userdata_type_metavalue<'a>(tab: &'a LuaAnyUserData<'a>) -> Option<String> {
    let s = tab
        .get_metatable()
        .ok()?
        .get::<LuaString>(LuaMetaMethod::Type.name())
        .ok()?;
    let s = s.to_str().ok()?;
    Some(s.to_string())
}

pub fn call_table_tostring_metamethod<'a>(tab: &'a LuaTable<'a>) -> Option<String> {
    tab.get_metatable()?
        .get::<_, LuaFunction>(LuaMetaMethod::ToString.name())
        .ok()?
        .call(tab)
        .ok()
}

pub fn call_userdata_tostring_metamethod<'a>(tab: &'a LuaAnyUserData<'a>) -> Option<String> {
    tab.get_metatable()
        .ok()?
        .get::<LuaFunction>(LuaMetaMethod::ToString.name())
        .ok()?
        .call(tab)
        .ok()
}
