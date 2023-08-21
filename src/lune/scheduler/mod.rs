use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    pin::Pin,
    sync::Arc,
};

use futures_util::{stream::FuturesUnordered, Future};
use mlua::prelude::*;
use tokio::sync::{
    broadcast::{channel, Sender},
    Mutex as AsyncMutex,
};

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
pub(crate) struct Scheduler<'fut> {
    state: Arc<SchedulerState>,
    threads: Arc<RefCell<VecDeque<SchedulerThread>>>,
    thread_senders: Arc<RefCell<HashMap<SchedulerThreadId, SchedulerThreadSender>>>,
    futures_lua: Arc<AsyncMutex<FuturesUnordered<SchedulerFuture<'fut>>>>,
    futures_background: Arc<AsyncMutex<FuturesUnordered<SchedulerFuture<'static>>>>,
    futures_break_signal: Sender<()>,
}

impl<'fut> Scheduler<'fut> {
    /**
        Creates a new scheduler.
    */
    pub fn new() -> Self {
        let (futures_break_signal, _) = channel(1);

        Self {
            state: Arc::new(SchedulerState::new()),
            threads: Arc::new(RefCell::new(VecDeque::new())),
            thread_senders: Arc::new(RefCell::new(HashMap::new())),
            futures_lua: Arc::new(AsyncMutex::new(FuturesUnordered::new())),
            futures_background: Arc::new(AsyncMutex::new(FuturesUnordered::new())),
            futures_break_signal,
        }
    }

    /**
        Sets the luau interrupt for this scheduler.

        This will propagate errors from any lua-spawned
        futures back to the lua threads that spawned them.
    */
    pub fn set_interrupt_for(&self, lua: &Lua) {
        // Propagate errors given to the scheduler back to their lua threads
        // FUTURE: Do profiling and anything else we need inside of this interrupt
        let state = self.state.clone();
        lua.set_interrupt(move |_| {
            if let Some(id) = state.get_current_thread_id() {
                if let Some(err) = state.get_thread_error(id) {
                    return Err(err);
                }
            }
            Ok(LuaVmState::Continue)
        });
    }

    /**
        Sets the exit code for the scheduler.

        This will stop the scheduler from resuming any more lua threads or futures.

        Panics if the exit code is set more than once.
    */
    pub fn set_exit_code(&self, code: impl Into<u8>) {
        assert!(
            self.state.exit_code().is_none(),
            "Exit code may only be set exactly once"
        );
        self.state.set_exit_code(code.into())
    }

    #[doc(hidden)]
    pub fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }
}
