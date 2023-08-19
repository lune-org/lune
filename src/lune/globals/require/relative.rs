use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    _lua: &'lua Lua,
    _ctx: &'ctx RequireContext,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    Err(LuaError::runtime(format!(
        "TODO: Support require for absolute paths (tried to require '{path}')"
    )))
}
