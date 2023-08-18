use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    pin::Pin,
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

    This wraps a [`Lua`] struct and exposes it as the `lua` property.
*/
#[derive(Debug)]
pub(crate) struct Scheduler<'fut> {
    pub(crate) lua: Lua,
    state: SchedulerState,
    threads: RefCell<VecDeque<SchedulerThread>>,
    thread_senders: RefCell<HashMap<SchedulerThreadId, SchedulerThreadSender>>,
    futures: AsyncMutex<FuturesUnordered<Pin<Box<dyn Future<Output = ()> + 'fut>>>>,
}

impl<'fut> Scheduler<'fut> {
    pub fn new() -> Self {
        let lua = Lua::new();

        Self {
            lua,
            state: SchedulerState::new(),
            threads: RefCell::new(VecDeque::new()),
            thread_senders: RefCell::new(HashMap::new()),
            futures: AsyncMutex::new(FuturesUnordered::new()),
        }
    }
}
