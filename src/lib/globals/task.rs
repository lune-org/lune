// TODO: Figure out a good way to remove all the shared boilerplate from these functions

use std::{
    sync::Weak,
    time::{Duration, Instant},
};

use mlua::prelude::*;
use smol::prelude::*;
use smol::{channel::Sender, LocalExecutor, Timer};

use crate::{utils::table_builder::TableBuilder, LuneMessage};

pub fn create(lua: &Lua) -> LuaResult<()> {
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

async fn run_registered_task(
    lua: &Lua,
    to_run: impl Future<Output = LuaResult<()>> + 'static,
    run_in_background: bool,
) -> LuaResult<()> {
    // Fetch global references to task executor & message sender
    let exec = lua
        .app_data_ref::<Weak<LocalExecutor>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let sender = lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    // Send a message that we have started our task
    sender
        .send(LuneMessage::Spawned)
        .await
        .map_err(LuaError::external)?;
    // Run the new task separately from the current one using the executor
    // FIXME: This should run the task instantly and only stop at the first yield
    let sender = sender.clone();
    let task = exec.spawn(async move {
        sender
            .send(match to_run.await {
                Ok(_) => LuneMessage::Finished,
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    });
    // Wait for the task to complete OR let it run in the background
    // Any lua errors will be sent through the message channel back
    // to the main thread which will then handle them properly
    if run_in_background {
        task.detach();
        Ok(())
    } else {
        task.await.map_err(LuaError::external)
    }
}

async fn task_cancel<'a>(lua: &'a Lua, thread: LuaThread<'a>) -> LuaResult<()> {
    let coroutine: LuaTable = lua.globals().raw_get("coroutine")?;
    let close: LuaFunction = coroutine.raw_get("close")?;
    close.call_async(thread).await?;
    Ok(())
}

async fn task_defer<'a>(lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Spawn a new detached task using a lua reference that we can use inside of our task
    let task_lua = lua.app_data_ref::<Weak<Lua>>().unwrap().upgrade().unwrap();
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(
        lua,
        async move {
            task_wait(&task_lua, None).await?;
            let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok(())
        },
        true,
    )
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
    run_registered_task(
        lua,
        async move {
            task_wait(&task_lua, duration).await?;
            let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok(())
        },
        true,
    )
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_spawn<'a>(lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Spawn a new detached task using a lua reference that we can use inside of our task
    let task_lua = lua.app_data_ref::<Weak<Lua>>().unwrap().upgrade().unwrap();
    let task_thread = tof_to_thread(lua, tof)?;
    let task_thread_key = lua.create_registry_value(task_thread)?;
    let lua_thread_to_return = lua.registry_value(&task_thread_key)?;
    run_registered_task(
        lua,
        async move {
            let thread = task_lua.registry_value::<LuaThread>(&task_thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok(())
        },
        true,
    )
    .await?;
    Ok(lua_thread_to_return)
}

async fn task_wait(lua: &Lua, duration: Option<f32>) -> LuaResult<f32> {
    let start = Instant::now();
    run_registered_task(
        lua,
        async move {
            Timer::after(
                duration
                    .map(Duration::from_secs_f32)
                    .unwrap_or(Duration::ZERO),
            )
            .await;
            Ok(())
        },
        false,
    )
    .await?;
    let end = Instant::now();
    Ok((end - start).as_secs_f32())
}
