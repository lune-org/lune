use mlua::prelude::*;

pub mod datatypes;
pub mod document;
pub mod instance;

#[cfg(test)]
mod tests;

fn make_dt<F>(lua: &Lua, f: F) -> LuaResult<LuaTable>
where
    F: Fn(&Lua, &LuaTable) -> LuaResult<()>,
{
    let tab = lua.create_table()?;
    f(lua, &tab)?;
    tab.set_readonly(true);
    Ok(tab)
}

#[rustfmt::skip]
fn make_all_datatypes(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaTable)>> {
	use datatypes::types::*;
    Ok(vec![
        ("UDim",         make_dt(lua, UDim::make_table)?),
        ("UDim2",        make_dt(lua, UDim2::make_table)?),
        ("Vector2",      make_dt(lua, Vector2::make_table)?),
        ("Vector2int16", make_dt(lua, Vector2int16::make_table)?),
        ("Vector3",      make_dt(lua, Vector3::make_table)?),
        ("Vector3int16", make_dt(lua, Vector3int16::make_table)?),
    ])
}

pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    for (name, tab) in make_all_datatypes(lua)? {
        exports.set(name, tab)?;
    }
    Ok(exports)
}
