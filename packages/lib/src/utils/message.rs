use mlua::prelude::*;

#[derive(Debug, Clone)]
pub enum LuneMessage {
    Exit(u8),
    Spawned,
    Finished,
    LuaError(LuaError),
}
