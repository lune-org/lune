use std::time::{Duration, Instant};

use mlua::prelude::*;
use smol::Timer;

use crate::utils::table_builder::TableBuilder;

pub async fn create(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set(
        "task",
        TableBuilder::new(lua)?
            .with_async_function("cancel", task_cancel)?
            .with_async_function("defer", task_defer)?
            .with_async_function("delay", task_delay)?
            .with_async_function("spawn", task_spawn)?
            .with_async_function("wait", task_wait)?
            .build_readonly()?,
    )
}

fn get_or_create_thread_from_arg<'a>(lua: &'a Lua, arg: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    match arg {
        LuaValue::Thread(thread) => Ok(thread),
        LuaValue::Function(func) => Ok(lua.create_thread(func)?),
        val => Err(LuaError::RuntimeError(format!(
            "Expected type thread or function, got {}",
            val.type_name()
        ))),
    }
}

async fn resume_thread(lua: &Lua, thread: LuaThread<'_>, args: LuaMultiValue<'_>) -> LuaResult<()> {
    let coroutine: LuaTable = lua.globals().raw_get("coroutine")?;
    let resume: LuaFunction = coroutine.raw_get("resume")?;
    // FIXME: This is blocking, we should spawn a local tokio task,
    // but doing that moves "thread" and "args", that both have
    // the lifetime of the outer function, so it doesn't work
    resume.call_async((thread, args)).await?;
    Ok(())
}

async fn task_cancel(lua: &Lua, thread: LuaThread<'_>) -> LuaResult<()> {
    let coroutine: LuaTable = lua.globals().raw_get("coroutine")?;
    let close: LuaFunction = coroutine.raw_get("close")?;
    close.call_async(thread).await?;
    Ok(())
}

async fn task_defer<'a>(
    lua: &'a Lua,
    (tof, args): (LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    // TODO: Defer (sleep a minimum amount of time)
    let thread = get_or_create_thread_from_arg(lua, tof)?;
    resume_thread(lua, thread.clone(), args).await?;
    Ok(thread)
}

async fn task_delay<'a>(
    lua: &'a Lua,
    (_delay, tof, args): (Option<f32>, LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    // TODO: Delay by the amount of time wanted
    let thread = get_or_create_thread_from_arg(lua, tof)?;
    resume_thread(lua, thread.clone(), args).await?;
    Ok(thread)
}

async fn task_spawn<'a>(
    lua: &'a Lua,
    (tof, args): (LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    let thread = get_or_create_thread_from_arg(lua, tof)?;
    resume_thread(lua, thread.clone(), args).await?;
    Ok(thread)
}

// FIXME: It doesn't seem possible to properly make an async wait
// function with mlua right now, something breaks when using
// the async wait function inside of a coroutine
async fn task_wait(_: &Lua, duration: Option<f32>) -> LuaResult<f32> {
    let start = Instant::now();
    Timer::after(
        duration
            .map(Duration::from_secs_f32)
            .unwrap_or(Duration::ZERO),
    )
    .await;
    let end = Instant::now();
    Ok((end - start).as_secs_f32())
}
