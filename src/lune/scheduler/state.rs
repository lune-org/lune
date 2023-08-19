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
    thread_id: RefCell<Option<SchedulerThreadId>>,
    thread_errors: RefCell<HashMap<SchedulerThreadId, LuaError>>,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_error_count(&self) {
        self.num_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn has_errored(&self) -> bool {
        self.num_errors.load(Ordering::SeqCst) > 0
    }

    pub fn exit_code(&self) -> Option<u8> {
        if self.exit_state.load(Ordering::SeqCst) {
            Some(self.exit_code.load(Ordering::SeqCst))
        } else {
            None
        }
    }

    pub fn has_exit_code(&self) -> bool {
        self.exit_state.load(Ordering::SeqCst)
    }

    pub fn set_exit_code(&self, code: impl Into<u8>) {
        self.exit_state.store(true, Ordering::SeqCst);
        self.exit_code.store(code.into(), Ordering::SeqCst);
    }

    pub fn get_current_thread_id(&self) -> Option<SchedulerThreadId> {
        *self.thread_id.borrow()
    }

    pub fn set_current_thread_id(&self, id: Option<SchedulerThreadId>) {
        self.thread_id.replace(id);
        self.num_resumptions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_thread_error(&self, id: SchedulerThreadId) -> Option<LuaError> {
        self.thread_errors.borrow_mut().remove(&id)
    }

    pub fn set_thread_error(&self, id: SchedulerThreadId, err: LuaError) {
        self.thread_errors.borrow_mut().insert(id, err);
    }
}
