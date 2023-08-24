use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    _ctx: &'ctx RequireContext<'lua>,
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
