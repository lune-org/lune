use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use tokio::sync::Mutex as AsyncMutex;

use mlua::prelude::*;

#[derive(Debug, Clone)]
pub(super) struct TaskWaiterState<'fut> {
    rets: Option<LuaResult<LuaMultiValue<'fut>>>,
    waker: Option<Waker>,
}

impl<'fut> TaskWaiterState<'fut> {
    pub fn new() -> Arc<AsyncMutex<Self>> {
        Arc::new(AsyncMutex::new(TaskWaiterState {
            rets: None,
            waker: None,
        }))
    }

    pub fn finalize(&mut self, rets: LuaResult<LuaMultiValue<'fut>>) {
        self.rets = Some(rets);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

#[derive(Debug)]
pub(super) struct TaskWaiterFuture<'fut> {
    state: Arc<AsyncMutex<TaskWaiterState<'fut>>>,
}

impl<'fut> TaskWaiterFuture<'fut> {
    pub fn new(state: &Arc<AsyncMutex<TaskWaiterState<'fut>>>) -> Self {
        Self {
            state: Arc::clone(state),
        }
    }
}

impl<'fut> Clone for TaskWaiterFuture<'fut> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<'fut> Future for TaskWaiterFuture<'fut> {
    type Output = LuaResult<LuaMultiValue<'fut>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.state.try_lock().unwrap();
        if let Some(rets) = shared_state.rets.clone() {
            Poll::Ready(rets)
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
