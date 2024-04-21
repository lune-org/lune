#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

/**
    Creates the `roblox` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?.build_readonly()
}
