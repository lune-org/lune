use std::fmt::Debug;
use std::sync::Weak;

use mlua::prelude::*;
use smol::{channel::Sender, LocalExecutor};
use smol::{future::yield_now, prelude::*};

use crate::LuneMessage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskRunMode {
    Blocking,
    Instant,
    Deferred,
}

pub async fn run_registered_task<T>(
    lua: &Lua,
    mode: TaskRunMode,
    to_run: impl Future<Output = LuaResult<T>> + 'static,
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
    let sender = sender.clone();
    let task = exec.spawn(async move {
        if mode == TaskRunMode::Deferred {
            yield_now().await;
        }
        sender
            .send(match to_run.await {
                Ok(_) => LuneMessage::Finished,
                Err(LuaError::CoroutineInactive) => LuneMessage::Finished, // Task was canceled
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    });
    // Wait for the task to complete OR let it run in the background
    // Any lua errors will be sent through the message channel back
    // to the main thread which will then handle them properly
    if mode == TaskRunMode::Blocking {
        task.await.map_err(LuaError::external)?;
    } else {
        task.detach();
    }
    // Yield once right away to let the above spawned task start working
    // instantly, forcing it to run until completion or until it yields
    yield_now().await;
    Ok(())
}
