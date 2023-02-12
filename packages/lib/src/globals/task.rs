use std::time::{Duration, Instant};

use mlua::prelude::*;
use tokio::time;

use crate::utils::{
    table::TableBuilder,
    task::{run_registered_task, TaskRunMode},
};

const MINIMUM_WAIT_OR_DELAY_DURATION: f32 = 10.0 / 1_000.0; // 10ms

// TODO: We should probably keep track of all threads in a scheduler userdata
// that takes care of scheduling in a better way, and it should keep resuming
// threads until it encounters a delayed / waiting thread, then task:sleep
pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
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
    TableBuilder::new(lua)?
        .with_async_function("cancel", task_cancel)?
        .with_async_function("delay", task_delay)?
        .with_async_function("defer", task_defer)?
        .with_async_function("spawn", task_spawn)?
        .with_async_function("wait", task_wait)?
        .build_readonly()
}

fn tof_to_thread<'a>(
    lua: &'static Lua,
    thread_or_function: LuaValue<'a>,
) -> LuaResult<LuaThread<'a>> {
    match thread_or_function {
        LuaValue::Thread(t) => Ok(t),
        LuaValue::Function(f) => Ok(lua.create_thread(f)?),
        value => Err(LuaError::RuntimeError(format!(
            "Argument must be a thread or function, got {}",
            value.type_name()
        ))),
    }
}

async fn task_cancel<'a>(lua: &'static Lua, thread: LuaThread<'a>) -> LuaResult<()> {
    let close: LuaFunction = lua.named_registry_value("coroutine.close")?;
    close.call_async::<_, LuaMultiValue>(thread).await?;
    Ok(())
}

async fn task_defer<'a>(
    lua: &'static Lua,
    (tof, args): (LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let task_args_key = lua.create_registry_value(args.into_vec())?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(lua, TaskRunMode::Deferred, async move {
        let thread: LuaThread = lua.registry_value(&task_thread_key)?;
        let argsv: Vec<LuaValue> = lua.registry_value(&task_args_key)?;
        let args = LuaMultiValue::from_vec(argsv);
        if thread.status() == LuaThreadStatus::Resumable {
            let _: LuaMultiValue = thread.into_async(args).await?;
        }
        lua.remove_registry_value(task_thread_key)?;
        lua.remove_registry_value(task_args_key)?;
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_delay<'a>(
    lua: &'static Lua,
    (duration, tof, args): (Option<f32>, LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let task_args_key = lua.create_registry_value(args.into_vec())?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    let start = Instant::now();
    let dur = Duration::from_secs_f32(
        duration
            .map(|d| d.max(MINIMUM_WAIT_OR_DELAY_DURATION))
            .unwrap_or(MINIMUM_WAIT_OR_DELAY_DURATION),
    );
    run_registered_task(lua, TaskRunMode::Instant, async move {
        let thread: LuaThread = lua.registry_value(&task_thread_key)?;
        // NOTE: We are somewhat busy-waiting here, but we have to do this to make sure
        // that delayed+cancelled threads do not prevent the tokio runtime from finishing
        while thread.status() == LuaThreadStatus::Resumable && start.elapsed() < dur {
            time::sleep(Duration::from_millis(1)).await;
        }
        if thread.status() == LuaThreadStatus::Resumable {
            let argsv: Vec<LuaValue> = lua.registry_value(&task_args_key)?;
            let args = LuaMultiValue::from_vec(argsv);
            let _: LuaMultiValue = thread.into_async(args).await?;
        }
        lua.remove_registry_value(task_thread_key)?;
        lua.remove_registry_value(task_args_key)?;
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_spawn<'a>(
    lua: &'static Lua,
    (tof, args): (LuaValue<'a>, LuaMultiValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let task_args_key = lua.create_registry_value(args.into_vec())?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(lua, TaskRunMode::Instant, async move {
        let thread: LuaThread = lua.registry_value(&task_thread_key)?;
        let argsv: Vec<LuaValue> = lua.registry_value(&task_args_key)?;
        let args = LuaMultiValue::from_vec(argsv);
        if thread.status() == LuaThreadStatus::Resumable {
            let _: LuaMultiValue = thread.into_async(args).await?;
        }
        lua.remove_registry_value(task_thread_key)?;
        lua.remove_registry_value(task_args_key)?;
        Ok(())
    })
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_wait(lua: &'static Lua, duration: Option<f32>) -> LuaResult<f32> {
    let start = Instant::now();
    run_registered_task(lua, TaskRunMode::Blocking, async move {
        time::sleep(Duration::from_secs_f32(
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
