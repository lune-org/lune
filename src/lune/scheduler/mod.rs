use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    pin::Pin,
    sync::Arc,
};

use futures_util::{stream::FuturesUnordered, Future};
use mlua::prelude::*;
use tokio::sync::Mutex as AsyncMutex;

mod state;
mod thread;
mod traits;

mod impl_async;
mod impl_runner;
mod impl_threads;

pub use self::thread::SchedulerThreadId;
pub use self::traits::*;

use self::{
    state::SchedulerState,
    thread::{SchedulerThread, SchedulerThreadSender},
};

type SchedulerFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

/**
    Scheduler for Lua threads and futures.

    This scheduler can be cheaply cloned and the underlying state
    and data will remain unchanged and accessible from all clones.
*/
#[derive(Debug, Clone)]
pub(crate) struct Scheduler<'lua, 'fut> {
    lua: &'lua Lua,
    state: Arc<SchedulerState>,
    threads: Arc<RefCell<VecDeque<SchedulerThread>>>,
    thread_senders: Arc<RefCell<HashMap<SchedulerThreadId, SchedulerThreadSender>>>,
    futures: Arc<AsyncMutex<FuturesUnordered<SchedulerFuture<'fut>>>>,
}

impl<'lua, 'fut> Scheduler<'lua, 'fut> {
    pub fn new(lua: &'lua Lua) -> Self {
        Self {
            lua,
            state: Arc::new(SchedulerState::new()),
            threads: Arc::new(RefCell::new(VecDeque::new())),
            thread_senders: Arc::new(RefCell::new(HashMap::new())),
            futures: Arc::new(AsyncMutex::new(FuturesUnordered::new())),
        }
    }

    #[doc(hidden)]
    pub fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }

    #[doc(hidden)]
    pub unsafe fn from_static(lua: &'static Scheduler) -> Self {
        *Box::from_raw(lua as *const Scheduler as *mut Scheduler)
    }
}
