use mlua::prelude::*;

/// Internal message enum for the task scheduler, used to notify
/// futures to wake up and schedule their respective blocking tasks
#[derive(Debug, Clone)]
pub enum TaskSchedulerMessage {
    NewBlockingTaskReady,
    NewLuaErrorReady(LuaError),
    Spawned,
    Terminated(LuaResult<()>),
}
