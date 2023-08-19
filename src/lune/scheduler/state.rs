use std::{
    cell::RefCell,
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
};

use mlua::Error as LuaError;

use super::SchedulerThreadId;

#[derive(Debug, Default)]
pub struct SchedulerState {
    exit_state: AtomicBool,
    exit_code: AtomicU8,
    num_resumptions: AtomicUsize,
    num_errors: AtomicUsize,
    // TODO: Use Arc<Mutex<T>> to make these thread and borrow safe
    thread_id: RefCell<Option<SchedulerThreadId>>,
    thread_errors: RefCell<HashMap<SchedulerThreadId, LuaError>>,
}

impl SchedulerState {
    /**
        Creates a new scheduler state.

        This scheduler state uses atomic operations for everything
        except lua resumption errors, and is completely thread safe.
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
        *self.thread_id.borrow()
    }

    /**
        Sets the currently running lua scheduler thread id.

        This should be set to `Some(id)` just before resuming a lua
        thread, and `None` while no lua thread is being resumed.
    */
    pub fn set_current_thread_id(&self, id: Option<SchedulerThreadId>) {
        self.thread_id.replace(id);
        self.num_resumptions.fetch_add(1, Ordering::Relaxed);
    }

    /**
        Gets the [`LuaError`] (if any) for the given `id`.

        Note that this removes the error from the scheduler state completely.
    */
    pub fn get_thread_error(&self, id: SchedulerThreadId) -> Option<LuaError> {
        self.thread_errors.borrow_mut().remove(&id)
    }

    /**
        Sets a [`LuaError`] for the given `id`.

        Note that this will replace any already existing [`LuaError`].
    */
    pub fn set_thread_error(&self, id: SchedulerThreadId, err: LuaError) {
        self.thread_errors.borrow_mut().insert(id, err);
    }
}
