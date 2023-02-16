use async_trait::async_trait;

use futures_util::Future;
use mlua::prelude::*;

use super::super::{
    async_handle::TaskSchedulerAsyncHandle, message::TaskSchedulerMessage,
    scheduler::TaskReference, scheduler::TaskScheduler,
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
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>;
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
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        self.queue_async_task(thread, None, None, async move {
            match func(self.lua).await {
                Ok(res) => match res.to_lua_multi(self.lua) {
                    Ok(multi) => Ok(Some(multi)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })
    }
}
