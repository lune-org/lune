// TODO: Figure out a good way to remove all the shared boilerplate from these functions

use std::{
    sync::Weak,
    time::{Duration, Instant},
};

use mlua::prelude::*;
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

async fn task_cancel<'a>(lua: &'a Lua, thread: LuaThread<'a>) -> LuaResult<()> {
    let coroutine: LuaTable = lua.globals().raw_get("coroutine")?;
    let close: LuaFunction = coroutine.raw_get("close")?;
    close.call_async(thread).await?;
    Ok(())
}

async fn task_defer<'a>(task_lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Boilerplate to get arc-ed lua & async executor
    let lua = task_lua
        .app_data_ref::<Weak<Lua>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let exec = task_lua
        .app_data_ref::<Weak<LocalExecutor>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let sender = task_lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    // Spawn a new detached thread
    sender
        .send(LuneMessage::Spawned)
        .await
        .map_err(LuaError::external)?;
    let thread = tof_to_thread(&lua, tof)?;
    let thread_key = lua.create_registry_value(thread)?;
    let thread_to_return = task_lua.registry_value(&thread_key)?;
    let thread_sender = sender.clone();
    exec.spawn(async move {
        let result = async {
            task_wait(&lua, None).await?;
            let thread = lua.registry_value::<LuaThread>(&thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok::<_, LuaError>(())
        };
        thread_sender
            .send(match result.await {
                Ok(_) => LuneMessage::Finished,
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    })
    .detach();
    Ok(thread_to_return)
}

async fn task_delay<'a>(
    task_lua: &'a Lua,
    (duration, tof): (Option<f32>, LuaValue<'a>),
) -> LuaResult<LuaThread<'a>> {
    // Boilerplate to get arc-ed lua & async executor
    let lua = task_lua
        .app_data_ref::<Weak<Lua>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let exec = task_lua
        .app_data_ref::<Weak<LocalExecutor>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let sender = task_lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    // Spawn a new detached thread
    sender
        .send(LuneMessage::Spawned)
        .await
        .map_err(LuaError::external)?;
    let thread = tof_to_thread(&lua, tof)?;
    let thread_key = lua.create_registry_value(thread)?;
    let thread_to_return = task_lua.registry_value(&thread_key)?;
    let thread_sender = sender.clone();
    exec.spawn(async move {
        let result = async {
            task_wait(&lua, duration).await?;
            let thread = lua.registry_value::<LuaThread>(&thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok::<_, LuaError>(())
        };
        thread_sender
            .send(match result.await {
                Ok(_) => LuneMessage::Finished,
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    })
    .detach();
    Ok(thread_to_return)
}

async fn task_spawn<'a>(task_lua: &'a Lua, tof: LuaValue<'a>) -> LuaResult<LuaThread<'a>> {
    // Boilerplate to get arc-ed lua & async executor
    let lua = task_lua
        .app_data_ref::<Weak<Lua>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let exec = task_lua
        .app_data_ref::<Weak<LocalExecutor>>()
        .unwrap()
        .upgrade()
        .unwrap();
    let sender = task_lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    // Spawn a new detached thread
    sender
        .send(LuneMessage::Spawned)
        .await
        .map_err(LuaError::external)?;
    let thread = tof_to_thread(&lua, tof)?;
    let thread_key = lua.create_registry_value(thread)?;
    let thread_to_return = task_lua.registry_value(&thread_key)?;
    let thread_sender = sender.clone();
    // FIXME: This does not run the thread instantly
    exec.spawn(async move {
        let result = async {
            let thread = lua.registry_value::<LuaThread>(&thread_key)?;
            if thread.status() == LuaThreadStatus::Resumable {
                thread.into_async::<_, LuaMultiValue>(()).await?;
            }
            Ok::<_, LuaError>(())
        };
        thread_sender
            .send(match result.await {
                Ok(_) => LuneMessage::Finished,
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    })
    .detach();
    Ok(thread_to_return)
}

async fn task_wait(lua: &Lua, duration: Option<f32>) -> LuaResult<f32> {
    let sender = lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    sender
        .send(LuneMessage::Spawned)
        .await
        .map_err(LuaError::external)?;
    let start = Instant::now();
    Timer::after(
        duration
            .map(Duration::from_secs_f32)
            .unwrap_or(Duration::ZERO),
    )
    .await;
    let end = Instant::now();
    sender
        .send(LuneMessage::Finished)
        .await
        .map_err(LuaError::external)?;
    Ok((end - start).as_secs_f32())
}
