use mlua::prelude::*;

use crate::lua::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let mut roblox_constants = Vec::new();
    let roblox_module = lune_roblox::module(lua)?;
    for pair in roblox_module.pairs::<LuaValue, LuaValue>() {
        roblox_constants.push(pair?);
    }
    // TODO: Add async functions for reading & writing documents, creating instances
    TableBuilder::new(lua)?
        .with_values(roblox_constants)?
        .build_readonly()
}
