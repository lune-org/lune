#![allow(clippy::module_inception)]

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

mod captures;
mod matches;
mod regex;

use self::regex::LuaRegex;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("new", new_regex)?
        .build_readonly()
}

fn new_regex(_: &Lua, pattern: String) -> LuaResult<LuaRegex> {
    LuaRegex::new(pattern)
}
