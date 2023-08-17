use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct SchedulerState {
    exit_state: AtomicBool,
    exit_code: AtomicU8,
    num_resumptions: AtomicUsize,
    num_errors: AtomicUsize,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resumption(&self) {
        self.num_resumptions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_error(&self) {
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
}
