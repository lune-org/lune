use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    ctx: &'ctx RequireContext<'lua>,
    source: &str,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let (abs_path, rel_path) = ctx.resolve_paths(source, path)?;
    if ctx.is_cached(&abs_path)? {
        ctx.get_from_cache(&abs_path)
    } else if ctx.is_pending(&abs_path)? {
        ctx.wait_for_cache(&abs_path).await
    } else {
        ctx.load_with_caching(&abs_path, &rel_path).await
    }
}