use core::panic;

use mlua::prelude::*;

use tokio::sync::mpsc;

use super::message::TaskSchedulerMessage;

/**
    A handle to a registered asynchronous background task.

    [`TaskSchedulerAsyncHandle::unregister`] must be
    called upon completion of the background task to
    prevent the task scheduler from running indefinitely.
*/
#[must_use = "Background tasks must be unregistered"]
#[derive(Debug)]
pub struct TaskSchedulerAsyncHandle {
    unregistered: bool,
    sender: mpsc::UnboundedSender<TaskSchedulerMessage>,
}

impl TaskSchedulerAsyncHandle {
    pub fn new(sender: mpsc::UnboundedSender<TaskSchedulerMessage>) -> Self {
        Self {
            unregistered: false,
            sender,
        }
    }

    pub fn unregister(mut self, result: LuaResult<()>) {
        self.unregistered = true;
        self.sender
            .send(TaskSchedulerMessage::Terminated(result))
            .unwrap_or_else(|_| {
                panic!(
                    "\
                    \nFailed to unregister background task - this is an internal error! \
                    \nPlease report it at {} \
                    \nDetails: Manual \
                    ",
                    env!("CARGO_PKG_REPOSITORY")
                )
            });
    }
}
