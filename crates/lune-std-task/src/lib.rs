#![allow(clippy::cargo_common_metadata)]

use std::time::Duration;

use mlua::prelude::*;
use mlua_luau_scheduler::Functions;

use tokio::{
    task::yield_now,
    time::{sleep, Instant},
};

use lune_utils::TableBuilder;

/**
    Creates the `task` standard library module.

    # Errors

    Errors when out of memory, or if default Lua globals are missing.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    let fns = Functions::new(lua.clone())?;

    // Create wait & delay functions
    let task_wait = lua.create_async_function(wait)?;
    let task_delay_env = TableBuilder::new(lua.clone())?
        .with_value("select", lua.globals().get::<LuaFunction>("select")?)?
        .with_value("spawn", fns.spawn.clone())?
        .with_value("defer", fns.defer.clone())?
        .with_value("wait", task_wait.clone())?
        .build_readonly()?;
    let task_delay = lua
        .load(DELAY_IMPL_LUA)
        .set_name("task.delay")
        .set_environment(task_delay_env)
        .into_function()?;

    TableBuilder::new(lua)?
        .with_value("cancel", fns.cancel)?
        .with_value("defer", fns.defer)?
        .with_value("delay", task_delay)?
        .with_value("spawn", fns.spawn)?
        .with_value("wait", task_wait)?
        .build_readonly()
}

const DELAY_IMPL_LUA: &str = r"
return defer(function(...)
    wait(select(1, ...))
    spawn(select(2, ...))
end, ...)
";

async fn wait(lua: Lua, secs: Option<f64>) -> LuaResult<f64> {
    // NOTE: We must guarantee that the task.wait API always yields
    // from a lua perspective, even if sleep/timer completes instantly
    yield_now().await;
    wait_inner(lua, secs).await
}

async fn wait_inner(_: Lua, secs: Option<f64>) -> LuaResult<f64> {
    let duration = Duration::from_secs_f64(secs.unwrap_or_default());

    let before = Instant::now();
    sleep(duration).await;
    let after = Instant::now();

    Ok((after - before).as_secs_f64())
}
