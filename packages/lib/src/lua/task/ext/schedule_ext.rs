use std::time::Duration;

use mlua::prelude::*;
use tokio::time::sleep;

use super::super::{scheduler::TaskKind, scheduler::TaskReference, scheduler::TaskScheduler};

/*
    ──────────────────────────────────────────────────────────
    Trait definition - same as the implementation, ignore this

    We use traits here to prevent misuse of certain scheduler
    APIs, making importing of them as intentional as possible
    ──────────────────────────────────────────────────────────
*/
pub trait TaskSchedulerScheduleExt {
    fn schedule_blocking(
        &self,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference>;

    fn schedule_blocking_deferred(
        &self,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference>;

    fn schedule_blocking_after_seconds(
        &self,
        after_secs: f64,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference>;
}

/*
    ────────────────────
    Trait implementation
    ────────────────────
*/
impl TaskSchedulerScheduleExt for TaskScheduler<'_> {
    /**
        Schedules a lua thread or function to resume ***first*** during this
        resumption point, ***skipping ahead*** of any other currently queued tasks.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    fn schedule_blocking(
        &self,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_blocking_task(TaskKind::Instant, thread, Some(thread_args), None)
    }

    /**
        Schedules a lua thread or function to resume ***after all***
        currently resuming tasks, during this resumption point.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    fn schedule_blocking_deferred(
        &self,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_blocking_task(TaskKind::Deferred, thread, Some(thread_args), None)
    }

    /**
        Schedules a lua thread or function to
        be resumed after waiting asynchronously.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    fn schedule_blocking_after_seconds(
        &self,
        after_secs: f64,
        thread: LuaThread<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_async_task(thread, Some(thread_args), None, async move {
            sleep(Duration::from_secs_f64(after_secs)).await;
            Ok(None)
        })
    }
}
