use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    name: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
    'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
{
    ctx.load_builtin(lua, name)
}
