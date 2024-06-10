use std::{cell::Cell, process::ExitCode, rc::Rc};

use event_listener::Event;

#[derive(Debug, Clone)]
pub(crate) struct Exit {
    code: Rc<Cell<Option<u8>>>,
    event: Rc<Event>,
}

impl Exit {
    pub fn new() -> Self {
        Self {
            code: Rc::new(Cell::new(None)),
            event: Rc::new(Event::new()),
        }
    }

    pub fn set(&self, code: u8) {
        self.code.set(Some(code));
        self.event.notify(usize::MAX);
    }

    pub fn get(&self) -> Option<u8> {
        self.code.get()
    }

    pub async fn listen(&self) {
        self.event.listen().await;
    }
}
