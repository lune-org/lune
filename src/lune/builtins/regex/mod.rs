use mlua::prelude::*;

use crate::lune::util::TableBuilder;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?.build_readonly()
}
