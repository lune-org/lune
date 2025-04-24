#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

mod captures;
mod matches;
mod regex;

use self::regex::LuaRegex;

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `regex` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `regex` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("new", new_regex)?
        .build_readonly()
}

fn new_regex(_: &Lua, pattern: String) -> LuaResult<LuaRegex> {
    LuaRegex::new(pattern)
}
