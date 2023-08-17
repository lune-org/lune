use std::cell::RefCell;

#[derive(Debug, Default)]
pub struct SchedulerState {
    exit_code: RefCell<Option<u8>>,
    num_resumptions: RefCell<usize>,
    num_errors: RefCell<usize>,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resumption(&self) {
        *self.num_resumptions.borrow_mut() += 1;
    }

    pub fn add_error(&self) {
        *self.num_errors.borrow_mut() += 1;
    }

    pub fn has_errored(&self) -> bool {
        *self.num_errors.borrow() > 0
    }

    pub fn exit_code(&self) -> Option<u8> {
        *self.exit_code.borrow()
    }

    pub fn has_exit_code(&self) -> bool {
        self.exit_code.borrow().is_some()
    }

    pub fn set_exit_code(&self, code: impl Into<u8>) {
        self.exit_code.replace(Some(code.into()));
    }
}
