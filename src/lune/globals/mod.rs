use mlua::prelude::*;

use super::util::TableBuilder;

mod g_table;
mod require;

pub fn inject_all(lua: &'static Lua) -> LuaResult<()> {
    let all = TableBuilder::new(lua)?
        .with_value("_G", g_table::create(lua)?)?
        .with_value("require", require::create(lua)?)?
        .build_readonly()?;

    for res in all.pairs() {
        let (key, value): (LuaValue, LuaValue) = res?;
        lua.globals().set(key, value)?;
    }

    Ok(())
}
