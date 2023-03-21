use std::time::Duration;

use async_trait::async_trait;

use mlua::prelude::*;

use futures_util::StreamExt;
use tokio::time::sleep;

use super::super::{
    scheduler_message::TaskSchedulerMessage, scheduler_state::TaskSchedulerState, TaskScheduler,
};

/*
    ──────────────────────────────────────────────────────────
    Trait definition - same as the implementation, ignore this

    We use traits here to prevent misuse of certain scheduler
    APIs, making importing of them as intentional as possible
    ──────────────────────────────────────────────────────────
*/
#[async_trait(?Send)]
pub trait TaskSchedulerResumeExt {
    async fn resume_queue(&self) -> TaskSchedulerState;
}

/*
    ────────────────────
    Trait implementation
    ────────────────────
*/
#[async_trait(?Send)]
impl TaskSchedulerResumeExt for TaskScheduler<'_> {
    /**
        Resumes the task scheduler queue.

        This will run any spawned or deferred Lua tasks in a blocking manner.

        Once all spawned and / or deferred Lua tasks have finished running,
        this will process delayed tasks, waiting tasks, and native Rust
        futures concurrently, awaiting the first one to be ready for resumption.
    */
    async fn resume_queue(&self) -> TaskSchedulerState {
        let current = TaskSchedulerState::new(self);
        let result = if current.num_blocking > 0 {
            // 1. Blocking tasks
            resume_next_blocking_task(self, None)
        } else if current.num_futures > 0 || current.num_background > 0 {
            // 2. Async and/or background tasks
            tokio::select! {
                result = resume_next_async_task(self) => result,
                result = receive_next_message(self) => result,
            }
        } else {
            // 3. No tasks left, here we sleep one millisecond in case
            // the caller of resume_queue accidentally calls this in
            // a busy loop to prevent cpu usage from going to 100%
            sleep(Duration::from_millis(1)).await;
            TaskSchedulerState::new(self)
        };
        result
    }
}

/*
    ────────────────────────────────────────────────────────────────
    Private functions for the trait that operate on the task scheduler

    These could be implemented as normal methods but if we put them in the
    trait they become public, and putting them in the task scheduler's
    own implementation block will clutter that up unnecessarily
    ────────────────────────────────────────────────────────────────
*/

/**
    Resumes the next queued Lua task, if one exists, blocking
    the current thread until it either yields or finishes.
*/
fn resume_next_blocking_task<'sched, 'args>(
    scheduler: &TaskScheduler<'sched>,
    override_args: Option<LuaResult<LuaMultiValue<'args>>>,
) -> TaskSchedulerState
where
    'args: 'sched,
{
    match {
        let mut queue_guard = scheduler.tasks_queue_blocking.borrow_mut();
        let task = queue_guard.pop_front();
        drop(queue_guard);
        task
    } {
        None => TaskSchedulerState::new(scheduler),
        Some(task) => match scheduler.resume_task(task, override_args) {
            Err(task_err) => {
                scheduler.wake_completed_task(task, Err(task_err.clone()));
                TaskSchedulerState::err(scheduler, task_err)
            }
            Ok(rets) if rets.0 == LuaThreadStatus::Unresumable => {
                scheduler.wake_completed_task(task, Ok(rets.1));
                TaskSchedulerState::new(scheduler)
            }
            Ok(_) => TaskSchedulerState::new(scheduler),
        },
    }
}

/**
    Awaits the first available queued future, and resumes its associated
    Lua task which will be ready for resumption when that future wakes.

    Panics if there are no futures currently queued.

    Use [`TaskScheduler::next_queue_future_exists`]
    to check if there are any queued futures.
*/
async fn resume_next_async_task(scheduler: &TaskScheduler<'_>) -> TaskSchedulerState {
    let (task, result) = {
        let mut futs = scheduler
            .futures
            .try_lock()
            .expect("Tried to resume next queued future while already resuming or modifying");
        futs.next()
            .await
            .expect("Tried to resume next queued future but none are queued")
    };
    // The future might not return a reference that it wants to resume
    if let Some(task) = task {
        // Promote this future task to a blocking task and resume it
        // right away, also taking care to not borrow mutably twice
        // by dropping this guard before trying to resume it
        let mut queue_guard = scheduler.tasks_queue_blocking.borrow_mut();
        queue_guard.push_front(task);
        drop(queue_guard);
    }
    resume_next_blocking_task(scheduler, result.transpose())
}

/**
    Awaits the next background task registration
    message, if any messages exist in the queue.

    This is a no-op if there are no background tasks left running
    and / or the background task messages channel was closed.
*/
async fn receive_next_message(scheduler: &TaskScheduler<'_>) -> TaskSchedulerState {
    let message_opt = {
        let mut rx = scheduler.futures_rx.lock().await;
        rx.recv().await
    };
    if let Some(message) = message_opt {
        match message {
            TaskSchedulerMessage::NewBlockingTaskReady => TaskSchedulerState::new(scheduler),
            TaskSchedulerMessage::NewLuaErrorReady(err) => TaskSchedulerState::err(scheduler, err),
            TaskSchedulerMessage::Spawned => {
                let prev = scheduler.futures_background_count.get();
                scheduler.futures_background_count.set(prev + 1);
                TaskSchedulerState::new(scheduler)
            }
            TaskSchedulerMessage::Terminated(result) => {
                let prev = scheduler.futures_background_count.get();
                scheduler.futures_background_count.set(prev - 1);
                if prev == 0 {
                    panic!(
                        r#"
                        Terminated a background task without it running - this is an internal error!
                        Please report it at {}
                        "#,
                        env!("CARGO_PKG_REPOSITORY")
                    )
                }
                if let Err(e) = result {
                    TaskSchedulerState::err(scheduler, e)
                } else {
                    TaskSchedulerState::new(scheduler)
                }
            }
        }
    } else {
        TaskSchedulerState::new(scheduler)
    }
}
