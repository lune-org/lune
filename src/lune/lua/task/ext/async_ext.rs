use std::time::Duration;

use async_trait::async_trait;

use futures_util::Future;
use mlua::prelude::*;
use tokio::time::{sleep, Instant};

use crate::lune::lua::task::TaskKind;

use super::super::{
    scheduler::TaskReference, scheduler::TaskScheduler, scheduler_handle::TaskSchedulerAsyncHandle,
    scheduler_message::TaskSchedulerMessage,
};

/*
    ──────────────────────────────────────────────────────────
    Trait definition - same as the implementation, ignore this

    We use traits here to prevent misuse of certain scheduler
    APIs, making importing of them as intentional as possible
    ──────────────────────────────────────────────────────────
*/
#[async_trait(?Send)]
pub trait TaskSchedulerAsyncExt<'fut> {
    fn register_background_task(&self) -> TaskSchedulerAsyncHandle;

    fn schedule_async<'sched, R, F, FR>(
        &'sched self,
        thread: LuaThread<'_>,
        func: F,
    ) -> LuaResult<TaskReference>
    where
        'sched: 'fut,
        R: IntoLuaMulti<'static>,
        F: 'static + Fn(&'static Lua) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>;

    fn schedule_wait(
        &'fut self,
        reference: LuaThread<'_>,
        duration: Option<f64>,
    ) -> LuaResult<TaskReference>;
}

/*
    ────────────────────
    Trait implementation
    ────────────────────
*/
#[async_trait(?Send)]
impl<'fut> TaskSchedulerAsyncExt<'fut> for TaskScheduler<'fut> {
    /**
        Registers a new background task with the task scheduler.

        The returned [`TaskSchedulerAsyncHandle`] must have its
        [`TaskSchedulerAsyncHandle::unregister`] method called
        upon completion of the background task to prevent
        the task scheduler from running indefinitely.
    */
    fn register_background_task(&self) -> TaskSchedulerAsyncHandle {
        let sender = self.futures_tx.clone();
        sender
            .send(TaskSchedulerMessage::Spawned)
            .unwrap_or_else(|e| {
                panic!(
                    "\
                    \nFailed to unregister background task - this is an internal error! \
                    \nPlease report it at {} \
                    \nDetails: {e} \
                    ",
                    env!("CARGO_PKG_REPOSITORY")
                )
            });
        TaskSchedulerAsyncHandle::new(sender)
    }

    /**
        Schedules a lua thread or function
        to be resumed after running a future.

        The given lua thread or function will be resumed
        using the optional arguments returned by the future.
    */
    fn schedule_async<'sched, R, F, FR>(
        &'sched self,
        thread: LuaThread<'_>,
        func: F,
    ) -> LuaResult<TaskReference>
    where
        'sched: 'fut, // Scheduler must live at least as long as the future
        R: IntoLuaMulti<'static>,
        F: 'static + Fn(&'static Lua) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        self.queue_async_task(thread, None, async move {
            match func(self.lua).await {
                Ok(res) => match res.into_lua_multi(self.lua) {
                    Ok(multi) => Ok(Some(multi)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })
    }

    /**
        Schedules a task reference to be resumed after a certain amount of time.

        The given task will be resumed with the elapsed time as its one and only argument.
    */
    fn schedule_wait(
        &'fut self,
        thread: LuaThread<'_>,
        duration: Option<f64>,
    ) -> LuaResult<TaskReference> {
        let reference = self.create_task(TaskKind::Future, thread, None, true)?;
        // Insert the future
        let futs = self
            .futures
            .try_lock()
            .expect("Tried to add future to queue during futures resumption");
        futs.push(Box::pin(async move {
            let before = Instant::now();
            sleep(Duration::from_secs_f64(
                duration.unwrap_or_default().max(0.0),
            ))
            .await;
            let elapsed_secs = before.elapsed().as_secs_f64();
            let args = elapsed_secs.into_lua_multi(self.lua).unwrap();
            (Some(reference), Ok(Some(args)))
        }));
        Ok(reference)
    }
}
