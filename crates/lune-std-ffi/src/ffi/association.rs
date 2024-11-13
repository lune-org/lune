use mlua::prelude::*;

// This is a small library that helps you set the dependencies of data in Lua.
// In FFI, there is often data that is dependent on other data.
// However, if you use user_value to inform Lua of the dependency,
// a table will be created for each userdata.
// To prevent this, we place a weak reference table in the named registry
// and simulate what mlua does.

// If the dependency is deep, the value may be completely destroyed when
// gc is performed multiple times. To prevent this situation, FFI should copy
// dependency if possible.

// You can delete the relationship by changing 'associated' to nil
#[inline]
pub fn set<'lua, T, U>(lua: &'lua Lua, regname: &str, value: T, associated: U) -> LuaResult<()>
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

// Returns the Lua value that 'value' keeps.
// If there is no table in registry, it returns None.
// If there is no value in table, it returns LuaNil.
#[inline]
pub fn get<'lua, T>(lua: &'lua Lua, regname: &str, value: T) -> LuaResult<Option<LuaValue<'lua>>>
where
    T: IntoLua<'lua>,
{
    match lua.named_registry_value::<LuaValue>(regname)? {
        LuaValue::Nil => Ok(None),
        LuaValue::Table(t) => Ok(Some(t.get(value)?)),
        _ => panic!(),
    }
}
