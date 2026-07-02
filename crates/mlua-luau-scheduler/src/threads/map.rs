#![allow(clippy::inline_always)]

use std::{cell::RefCell, rc::Rc};

use mlua::prelude::*;
use rustc_hash::FxHashMap;

use crate::events::{OnceEvent, OnceListener};

use super::id::ThreadId;

struct ThreadEvent {
    result: Option<LuaResult<LuaMultiValue>>,
    event: OnceEvent,
}

impl ThreadEvent {
    fn new() -> Self {
        Self {
            result: None,
            event: OnceEvent::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ThreadMap {
    inner: Rc<RefCell<FxHashMap<ThreadId, ThreadEvent>>>,
}

impl ThreadMap {
    pub fn new() -> Self {
        let inner = Rc::new(RefCell::new(FxHashMap::default()));
        Self { inner }
    }

    #[inline(always)]
    pub fn track(&self, id: ThreadId) {
        self.inner.borrow_mut().insert(id, ThreadEvent::new());
    }

    #[inline(always)]
    pub fn is_tracked(&self, id: ThreadId) -> bool {
        self.inner.borrow().contains_key(&id)
    }

    #[inline(always)]
    pub fn insert(&self, id: ThreadId, result: LuaResult<LuaMultiValue>) {
        if let Some(tracker) = self.inner.borrow_mut().get_mut(&id) {
            tracker.result.replace(result);
            tracker.event.notify();
        } else {
            panic!("Thread must be tracked");
        }
    }

    #[inline(always)]
    pub fn listen(&self, id: ThreadId) -> OnceListener {
        if let Some(tracker) = self.inner.borrow().get(&id) {
            tracker.event.listen()
        } else {
            panic!("Thread must be tracked");
        }
    }

    #[inline(always)]
    pub fn remove(&self, id: ThreadId) -> Option<LuaResult<LuaMultiValue>> {
        if let Some(mut tracker) = self.inner.borrow_mut().remove(&id) {
            tracker.result.take()
        } else {
            None
        }
    }
}
