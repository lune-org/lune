use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua>(
    _lua: &'lua Lua,
    _ctx: RequireContext,
    name: &str,
) -> LuaResult<LuaValue<'lua>> {
    Err(LuaError::runtime(format!(
        "TODO: Support require for built-in libraries (tried to require '{name}')"
    )))
}
