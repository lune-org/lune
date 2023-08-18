use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ops::Deref,
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

pub use self::traits::*;

use self::{
    state::SchedulerState,
    thread::{SchedulerThread, SchedulerThreadId, SchedulerThreadSender},
};

/**
    Scheduler for Lua threads.

    Can be cheaply cloned, and any clone will refer
    to the same underlying scheduler and Lua struct.
*/
#[derive(Debug, Clone)]
pub(crate) struct Scheduler<'fut> {
    inner: Arc<SchedulerImpl<'fut>>,
}

impl<'fut> Scheduler<'fut> {
    /**
        Creates a new scheduler for the given [`Lua`] struct.
    */
    pub fn new(lua: Arc<Lua>) -> Self {
        let sched_lua = Arc::clone(&lua);
        let sched_impl = SchedulerImpl::new(sched_lua);

        let inner = Arc::new(sched_impl);

        Self { inner }
    }
}

impl<'fut> Deref for Scheduler<'fut> {
    type Target = SchedulerImpl<'fut>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/**
    Implementation of scheduler for Lua threads.

    Not meant to be used directly, use [`Scheduler`] instead.
*/
#[derive(Debug)]
pub(crate) struct SchedulerImpl<'fut> {
    lua: Arc<Lua>,
    state: SchedulerState,
    threads: RefCell<VecDeque<SchedulerThread>>,
    thread_senders: RefCell<HashMap<SchedulerThreadId, SchedulerThreadSender>>,
    futures: AsyncMutex<FuturesUnordered<Pin<Box<dyn Future<Output = ()> + 'fut>>>>,
}

impl<'fut> SchedulerImpl<'fut> {
    fn new(lua: Arc<Lua>) -> Self {
        Self {
            lua,
            state: SchedulerState::new(),
            threads: RefCell::new(VecDeque::new()),
            thread_senders: RefCell::new(HashMap::new()),
            futures: AsyncMutex::new(FuturesUnordered::new()),
        }
    }
}
