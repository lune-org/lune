use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua>(
    _lua: &'lua Lua,
    _ctx: RequireContext,
    path: &str,
) -> LuaResult<LuaValue<'lua>> {
    Err(LuaError::runtime(format!(
        "TODO: Support require for absolute paths (tried to require '{path}')"
    )))
}
