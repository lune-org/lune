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
pub struct Scheduler(Arc<SchedulerImpl>);

impl Scheduler {
    /**
        Creates a new scheduler for the given [`Lua`] struct.
    */
    pub fn new(lua: Arc<Lua>) -> Self {
        assert!(
            lua.app_data_ref::<Self>().is_none() && lua.app_data_ref::<&Self>().is_none(),
            "Only one scheduler may be created per Lua struct"
        );

        let inner = SchedulerImpl::new(Arc::clone(&lua));
        let sched = Self(Arc::new(inner));

        lua.set_app_data(sched.clone());
        lua.set_interrupt(move |_| Ok(LuaVmState::Continue));

        sched
    }
}

impl Deref for Scheduler {
    type Target = SchedulerImpl;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/**
    Implementation of scheduler for Lua threads.

    Not meant to be used directly, use [`Scheduler`] instead.
*/
#[derive(Debug)]
pub struct SchedulerImpl {
    lua: Arc<Lua>,
    state: SchedulerState,
    threads: RefCell<VecDeque<SchedulerThread>>,
    thread_senders: RefCell<HashMap<SchedulerThreadId, SchedulerThreadSender>>,
    futures: AsyncMutex<FuturesUnordered<Pin<Box<dyn Future<Output = ()>>>>>,
}

impl SchedulerImpl {
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
