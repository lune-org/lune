use std::{cell::Cell, rc::Rc};

use crate::events::OnceEvent;

#[derive(Debug, Clone)]
pub(crate) struct Exit {
    code: Rc<Cell<Option<u8>>>,
    event: OnceEvent,
}

impl Exit {
    pub fn new() -> Self {
        Self {
            code: Rc::new(Cell::new(None)),
            event: OnceEvent::new(),
        }
    }

    pub fn set(&self, code: u8) {
        self.code.set(Some(code));
        self.event.notify();
    }

    pub fn get(&self) -> Option<u8> {
        self.code.get()
    }

    pub async fn listen(&self) {
        self.event.listen().await;
    }
}
