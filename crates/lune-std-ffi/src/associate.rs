#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

pub fn set_associate<'lua, T, U>(
    lua: &'lua Lua,
    regname: &str,
    value: T,
    associated: U,
) -> LuaResult<()>
where
    T: IntoLua<'lua>,
    U: IntoLua<'lua>,
{
    let table = match lua.named_registry_value::<LuaValue>(regname)? {
        LuaValue::Nil => {
            let table = lua.create_table()?;
            lua.set_named_registry_value(regname, table.clone())?;
            let meta = lua.create_table()?;
            meta.set("__mode", "k")?;
            table.set_metatable(Some(meta));
            table
        }
        LuaValue::Table(t) => t,
        _ => panic!(""),
    };

    table.set(value, associated)?;

    Ok(())
}

pub fn get_associate<'lua, T>(
    lua: &'lua Lua,
    regname: &str,
    value: T,
) -> LuaResult<Option<LuaValue<'lua>>>
where
    T: IntoLua<'lua>,
{
    match lua.named_registry_value::<LuaValue>(regname)? {
        LuaValue::Nil => Ok(None),
        LuaValue::Table(t) => Ok(Some(t.get(value)?)),
        _ => panic!(),
    }
}

pub fn get_table<'lua>(lua: &'lua Lua, regname: &str) -> LuaResult<Option<LuaTable<'lua>>> {
    match lua.named_registry_value::<LuaValue>(regname)? {
        LuaValue::Nil => Ok(None),
        LuaValue::Table(t) => Ok(Some(t)),
        _ => panic!(),
    }
}
