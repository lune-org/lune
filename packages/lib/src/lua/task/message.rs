use mlua::prelude::*;

#[derive(Debug, Clone)]
pub enum TaskSchedulerMessage {
    NewBlockingTaskReady,
    Spawned,
    Terminated(LuaResult<()>),
}
