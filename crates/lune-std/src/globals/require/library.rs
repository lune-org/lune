use mlua::prelude::*;

use super::context::*;

pub(super) fn require(lua: Lua, ctx: &RequireContext, name: &str) -> LuaResult<LuaMultiValue> {
    ctx.load_library(lua, name)
}
