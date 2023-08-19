use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    _lua: &'lua Lua,
    _ctx: &'ctx RequireContext,
    alias: &str,
    name: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    Err(LuaError::runtime(format!(
        "TODO: Support require for built-in libraries (tried to require '{name}' with alias '{alias}')"
    )))
}
