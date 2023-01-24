use std::{
    sync::Weak,
    time::{Duration, Instant},
};

use mlua::prelude::*;
use smol::Timer;

use crate::utils::{
    table::TableBuilder,
    task::{run_registered_task, TaskRunMode},
};

const MINIMUM_WAIT_OR_DELAY_DURATION: f32 = 10.0 / 1_000.0; // 10ms

pub fn create(lua: &Lua) -> LuaResult<()> {
    // HACK: There is no way to call coroutine.close directly from the mlua
    // crate, so we need to fetch the function and store it in the registry
    let coroutine: LuaTable = lua.globals().raw_get("coroutine")?;
    let close: LuaFunction = coroutine.raw_get("close")?;
    lua.set_named_registry_value("coroutine.close", close)?;
    // HACK: coroutine.resume has some weird scheduling issues, but our custom
    // task.spawn implementation is more or less a replacement for it, so we
    // overwrite the original coroutine.resume function with it to fix that
    coroutine.raw_set("resume", lua.create_async_function(task_spawn)?)?;
    // Rest of the task library is normal, just async functions, no metatable
    lua.globals().raw_set(
        "task",
        TableBuilder::new(lua)?
            .with_async_function("cancel", task_cancel)?
            .with_async_function("delay", task_delay)?
            .with_async_function("defer", task_defer)?
            .with_async_function("spawn", task_spawn)?
            .with_async_function("wait", task_wait)?
            .build_readonly()?,
    )
}

fn tof_to_thread<'a>(lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    match tof {
        LuaValue::Thread(t) => Ok(t),
        LuaValue::Function(f) => Ok(lua.create_thread(f)?),
        value => Err(LuaError::RuntimeError(format!(
            "Argument must be a thread or function, got {}",
            value.type_name()
        ))),
    }
}

async fn task_cancel<'a>(lua: &'a Lua, thread: LuaThread<'a>) -> LuaResult<()> {
    let close: LuaFunction = lua.named_registry_value("coroutine.close")?;
    close.call_async::<_, LuaMultiValue>(thread).await?;
    Ok(())
}

async fn task_defer<'a>(lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Spawn a new detached task using a lua reference that we can use inside of our task
    let task_lua = lua.app_data_ref::<Weak<Lua>>().unwrap().upgrade().unwrap();
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(lua, TaskRunMode::Deferred, async move {
        let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
        if thread.status() == LuaThreadStatus::Resumable {
            thread.into_async::<_, LuaMultiValue>(()).await?;
        }
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_delay<'a>(
    lua: &'a Lua,
    (duration, tof): (Option<f32>, LuaValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    // Spawn a new detached task using a lua reference that we can use inside of our task
    let task_lua = lua.app_data_ref::<Weak<Lua>>().unwrap().upgrade().unwrap();
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(lua, TaskRunMode::Deferred, async move {
        task_wait(&task_lua, duration).await?;
        let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
        if thread.status() == LuaThreadStatus::Resumable {
            thread.into_async::<_, LuaMultiValue>(()).await?;
        }
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_spawn<'a>(lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Spawn a new detached task using a lua reference that we can use inside of our task
    let task_lua = lua.app_data_ref::<Weak<Lua>>().unwrap().upgrade().unwrap();
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(lua, TaskRunMode::Instant, async move {
        let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
        if thread.status() == LuaThreadStatus::Resumable {
            thread.into_async::<_, LuaMultiValue>(()).await?;
        }
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_wait(lua: &Lua, duration: Option<f32>) -> LuaResult<f32> {
    let start = Instant::now();
    run_registered_task(lua, TaskRunMode::Blocking, async move {
        Timer::after(Duration::from_secs_f32(
            duration
                .map(|d| d.max(MINIMUM_WAIT_OR_DELAY_DURATION))
                .unwrap_or(MINIMUM_WAIT_OR_DELAY_DURATION),
        ))
        .await;
        Ok(())
    })
    .await?;
    let end = Instant::now();
    Ok((end - start).as_secs_f32())
}
