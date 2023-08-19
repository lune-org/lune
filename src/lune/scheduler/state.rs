use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use mlua::Error as LuaError;

use super::SchedulerThreadId;

/**
    Internal state for a [`Scheduler`].

    This scheduler state uses atomic operations for everything
    except lua error storage, and is completely thread safe.
*/
#[derive(Debug, Default)]
pub struct SchedulerState {
    exit_state: AtomicBool,
    exit_code: AtomicU8,
    num_resumptions: AtomicUsize,
    num_errors: AtomicUsize,
    thread_id: Arc<Mutex<Option<SchedulerThreadId>>>,
    thread_errors: Arc<Mutex<HashMap<SchedulerThreadId, LuaError>>>,
}

impl SchedulerState {
    /**
        Creates a new scheduler state.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Increments the total lua error count for the scheduler.

        This is used to determine if the scheduler should exit with
        a non-zero exit code, when no exit code is explicitly set.
    */
    pub fn increment_error_count(&self) {
        self.num_errors.fetch_add(1, Ordering::Relaxed);
    }

    /**
        Checks if there have been any lua errors.

        This is used to determine if the scheduler should exit with
        a non-zero exit code, when no exit code is explicitly set.
    */
    pub fn has_errored(&self) -> bool {
        self.num_errors.load(Ordering::SeqCst) > 0
    }

    /**
        Gets the currently set exit code for the scheduler, if any.
    */
    pub fn exit_code(&self) -> Option<u8> {
        if self.exit_state.load(Ordering::SeqCst) {
            Some(self.exit_code.load(Ordering::SeqCst))
        } else {
            None
        }
    }

    /**
        Checks if the scheduler has an explicit exit code set.
    */
    pub fn has_exit_code(&self) -> bool {
        self.exit_state.load(Ordering::SeqCst)
    }

    /**
        Sets the explicit exit code for the scheduler.
    */
    pub fn set_exit_code(&self, code: impl Into<u8>) {
        self.exit_state.store(true, Ordering::SeqCst);
        self.exit_code.store(code.into(), Ordering::SeqCst);
    }

    /**
        Gets the currently running lua scheduler thread id, if any.
    */
    pub fn get_current_thread_id(&self) -> Option<SchedulerThreadId> {
        *self
            .thread_id
            .lock()
            .expect("Failed to lock current thread id")
    }

    /**
        Sets the currently running lua scheduler thread id.

        This must be set to `Some(id)` just before resuming a lua thread,
        and `None` while no lua thread is being resumed. If set to `Some`
        while the current thread id is also `Some`, this will panic.

        Must only be set once per thread id, although this
        is not checked at runtime for performance reasons.
    */
    pub fn set_current_thread_id(&self, id: Option<SchedulerThreadId>) {
        self.num_resumptions.fetch_add(1, Ordering::Relaxed);
        let mut thread_id = self
            .thread_id
            .lock()
            .expect("Failed to lock current thread id");
        assert!(
            id.is_none() || thread_id.is_none(),
            "Current thread id can not be overwritten"
        );
        *thread_id = id;
    }

    /**
        Gets the [`LuaError`] (if any) for the given `id`.

        Note that this removes the error from the scheduler state completely.
    */
    pub fn get_thread_error(&self, id: SchedulerThreadId) -> Option<LuaError> {
        let mut thread_errors = self
            .thread_errors
            .lock()
            .expect("Failed to lock thread errors");
        thread_errors.remove(&id)
    }

    /**
        Sets a [`LuaError`] for the given `id`.

        Note that this will replace any already existing [`LuaError`].
    */
    pub fn set_thread_error(&self, id: SchedulerThreadId, err: LuaError) {
        let mut thread_errors = self
            .thread_errors
            .lock()
            .expect("Failed to lock thread errors");
        thread_errors.insert(id, err);
    }
}
