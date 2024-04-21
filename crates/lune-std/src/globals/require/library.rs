use mlua::prelude::*;

use super::context::*;

pub(super) fn require<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    name: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    ctx.load_library(lua, name)
}
