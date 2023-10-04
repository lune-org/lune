use mlua::prelude::*;

use super::util::TableBuilder;

mod g_table;
mod print;
mod require;
mod version;
mod warn;

pub fn inject_all(lua: &'static Lua) -> LuaResult<()> {
    let all = TableBuilder::new(lua)?
        .with_value("_G", g_table::create(lua)?)?
        .with_value("_VERSION", version::create(lua)?)?
        .with_value("print", print::create(lua)?)?
        .with_value("require", require::create(lua)?)?
        .with_value("warn", warn::create(lua)?)?
        .build_readonly()?;

    for res in all.pairs() {
        let (key, value): (LuaValue, LuaValue) = res.unwrap();
        lua.globals().set(key, value)?;
    }

    Ok(())
}
