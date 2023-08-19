use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    if ctx.is_cached(path)? {
        ctx.get_from_cache(lua, path)
    } else if ctx.is_pending(path)? {
        ctx.wait_for_cache(lua, path).await
    } else {
        ctx.load(lua, path).await
    }
}
