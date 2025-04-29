#![allow(clippy::inline_always)]

use std::{cell::RefCell, rc::Rc};

use event_listener::Event;
// NOTE: This is the hash algorithm that mlua also uses, so we
// are not adding any additional dependencies / bloat by using it.
use mlua::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::thread_id::ThreadId;

struct ThreadResultMapInner {
    tracked: FxHashSet<ThreadId>,
    results: FxHashMap<ThreadId, LuaResult<LuaMultiValue>>,
    events: FxHashMap<ThreadId, Rc<Event>>,
}

impl ThreadResultMapInner {
    fn new() -> Self {
        Self {
            tracked: FxHashSet::default(),
            results: FxHashMap::default(),
            events: FxHashMap::default(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ThreadResultMap {
    inner: Rc<RefCell<ThreadResultMapInner>>,
}

impl ThreadResultMap {
    pub fn new() -> Self {
        let inner = Rc::new(RefCell::new(ThreadResultMapInner::new()));
        Self { inner }
    }

    #[inline(always)]
    pub fn track(&self, id: ThreadId) {
        self.inner.borrow_mut().tracked.insert(id);
    }

    #[inline(always)]
    pub fn is_tracked(&self, id: ThreadId) -> bool {
        self.inner.borrow().tracked.contains(&id)
    }

    pub fn insert(&self, id: ThreadId, result: LuaResult<LuaMultiValue>) {
        debug_assert!(self.is_tracked(id), "Thread must be tracked");
        let mut inner = self.inner.borrow_mut();
        inner.results.insert(id, result);
        if let Some(event) = inner.events.remove(&id) {
            event.notify(usize::MAX);
        }
    }

    pub async fn listen(&self, id: ThreadId) {
        debug_assert!(self.is_tracked(id), "Thread must be tracked");
        if !self.inner.borrow().results.contains_key(&id) {
            let listener = {
                let mut inner = self.inner.borrow_mut();
                let event = inner
                    .events
                    .entry(id)
                    .or_insert_with(|| Rc::new(Event::new()));
                event.listen()
            };
            listener.await;
        }
    }

    pub fn remove(&self, id: ThreadId) -> Option<LuaResult<LuaMultiValue>> {
        let mut inner = self.inner.borrow_mut();
        let res = inner.results.remove(&id)?;
        inner.tracked.remove(&id);
        inner.events.remove(&id);
        Some(res)
    }
}
