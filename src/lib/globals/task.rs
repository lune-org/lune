use std::{thread::sleep, time::Duration};

use mlua::{Function, Lua, Result, Table, Value};

use crate::utils::table_builder::ReadonlyTableBuilder;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

const TASK_LIB_LUAU: &str = include_str!("../luau/task.luau");

pub async fn new(lua: &Lua) -> Result<Table> {
    let task_lib: Table = lua
        .load(TASK_LIB_LUAU)
        .set_name("task")?
        .eval_async()
        .await?;
    // FUTURE: Properly implementing the task library in async rust is
    // very complicated but should be done at some point, for now we will
    // fall back to implementing only task.wait and doing the rest in lua
    let task_cancel: Function = task_lib.raw_get("cancel")?;
    let task_defer: Function = task_lib.raw_get("defer")?;
    let task_delay: Function = task_lib.raw_get("delay")?;
    let task_spawn: Function = task_lib.raw_get("spawn")?;
    ReadonlyTableBuilder::new(lua)?
        .with_value("cancel", Value::Function(task_cancel))?
        .with_value("defer", Value::Function(task_defer))?
        .with_value("delay", Value::Function(task_delay))?
        .with_value("spawn", Value::Function(task_spawn))?
        .with_function("wait", wait)?
        .build()
}

// FIXME: It does seem possible to properly make an async wait
// function with mlua right now, something breaks when using
// async wait functions inside of coroutines
fn wait(_: &Lua, duration: Option<f32>) -> Result<f32> {
    let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
    sleep(Duration::from_secs_f32(secs));
    Ok(secs)
}
