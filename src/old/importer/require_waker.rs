use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use tokio::sync::Mutex as AsyncMutex;

use mlua::prelude::*;

#[derive(Debug, Clone)]
pub(super) struct RequireWakerState<'lua> {
    rets: Option<LuaResult<LuaMultiValue<'lua>>>,
    waker: Option<Waker>,
}

impl<'lua> RequireWakerState<'lua> {
    pub fn new() -> Arc<AsyncMutex<Self>> {
        Arc::new(AsyncMutex::new(RequireWakerState {
            rets: None,
            waker: None,
        }))
    }

    pub fn finalize(&mut self, rets: LuaResult<LuaMultiValue<'lua>>) {
        self.rets = Some(rets);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

#[derive(Debug)]
pub(super) struct RequireWakerFuture<'lua> {
    state: Arc<AsyncMutex<RequireWakerState<'lua>>>,
}

impl<'lua> RequireWakerFuture<'lua> {
    pub fn new(state: &Arc<AsyncMutex<RequireWakerState<'lua>>>) -> Self {
        Self {
            state: Arc::clone(state),
        }
    }
}

impl<'lua> Clone for RequireWakerFuture<'lua> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

impl<'lua> Future for RequireWakerFuture<'lua> {
    type Output = LuaResult<LuaMultiValue<'lua>>;
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
