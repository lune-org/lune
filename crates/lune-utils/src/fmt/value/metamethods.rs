use mlua::prelude::*;

pub fn call_table_tostring_metamethod<'a>(tab: &'a LuaTable<'a>) -> Option<String> {
    let f = match tab.get_metatable() {
        None => None,
        Some(meta) => match meta.get::<_, LuaFunction>(LuaMetaMethod::ToString.name()) {
            Ok(method) => Some(method),
            Err(_) => None,
        },
    }?;
    match f.call::<_, String>(()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

pub fn call_userdata_tostring_metamethod<'a>(tab: &'a LuaAnyUserData<'a>) -> Option<String> {
    let f = match tab.get_metatable() {
        Err(_) => None,
        Ok(meta) => match meta.get::<LuaFunction>(LuaMetaMethod::ToString.name()) {
            Ok(method) => Some(method),
            Err(_) => None,
        },
    }?;
    match f.call::<_, String>(()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}
