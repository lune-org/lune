#![allow(clippy::cargo_common_metadata)]

// This is a small library that helps you set the dependencies of data in Lua.
// In FFI, there is often data that is dependent on other data.
// However, if you use user_value to inform Lua of the dependency,
// a table will be created for each userdata.
// To prevent this, we place a weak reference table in the registry
// and simulate what mlua does.
// Since mlua does not provide Lua state (private),
// uservalue operations cannot be performed directly,
// so this is the best solution for now.

use mlua::prelude::*;

// Forces 'associated' to persist as long as 'value' is alive.
// 'value' can only hold one value. If you want to keep something else,
// use a table with a different name.
// You can delete the relationship by changing 'associated' to nil
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

// returns the Lua value that 'value' keeps.
// If there is no table in registry, it returns None.
// If there is no value in table, it returns LuaNil.
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

// Allows reading of registry tables for debugging.
// This helps keep track of data being gc'd.
// However, for security and safety reasons,
// this will not be allowed unless it is a debug build.
#[cfg(debug_assertions)]
pub fn get_table<'lua>(lua: &'lua Lua, regname: &str) -> LuaResult<Option<LuaTable<'lua>>> {
    match lua.named_registry_value::<LuaValue>(regname)? {
        LuaValue::Nil => Ok(None),
        LuaValue::Table(t) => Ok(Some(t)),
        _ => panic!(),
    }
}
