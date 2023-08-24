use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    ctx: &'ctx RequireContext<'lua>,
    name: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
    'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
{
    ctx.load_builtin(name)
}
