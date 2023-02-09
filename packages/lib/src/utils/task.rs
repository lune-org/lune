use std::fmt::{self, Debug};
use std::future::Future;
use std::sync::Weak;

use mlua::prelude::*;
use tokio::sync::mpsc::Sender;
use tokio::task;

use crate::utils::message::LuneMessage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskRunMode {
    Blocking,
    Instant,
    Deferred,
}

impl fmt::Display for TaskRunMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocking => write!(f, "Blocking"),
            Self::Instant => write!(f, "Instant"),
            Self::Deferred => write!(f, "Deferred"),
        }
    }
}

pub async fn run_registered_task<T>(
    lua: &Lua,
    mode: TaskRunMode,
    to_run: impl Future<Output = LuaResult<T>> + 'static,
) -> LuaResult<()> {
    // Fetch global reference to message sender
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
    let task = task::spawn_local(async move {
        // HACK: For deferred tasks we yield a bunch of times to try and ensure
        // we run our task at the very end of the async queue, this can fail if
        // the user creates a bunch of interleaved deferred and normal tasks
        if mode == TaskRunMode::Deferred {
            for _ in 0..64 {
                task::yield_now().await;
            }
        }
        sender
            .send(match to_run.await {
                Ok(_) => LuneMessage::Finished,
                Err(LuaError::CoroutineInactive) => LuneMessage::Finished, // Task was canceled
                Err(e) => LuneMessage::LuaError(e),
            })
            .await
    });
    // Wait for the task to complete if we want this call to be blocking
    // Any lua errors will be sent through the message channel back
    // to the main thread which will then handle them properly
    if mode == TaskRunMode::Blocking {
        task.await
            .map_err(LuaError::external)?
            .map_err(LuaError::external)?;
    }
    // Yield once right away to let the above spawned task start working
    // instantly, forcing it to run until completion or until it yields
    task::yield_now().await;
    Ok(())
}
