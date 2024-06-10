#![allow(clippy::inline_always)]

use std::{cell::RefCell, rc::Rc};

use event_listener::Event;
// NOTE: This is the hash algorithm that mlua also uses, so we
// are not adding any additional dependencies / bloat by using it.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{thread_id::ThreadId, util::ThreadResult};

#[derive(Clone)]
pub(crate) struct ThreadResultMap {
    tracked: Rc<RefCell<FxHashSet<ThreadId>>>,
    results: Rc<RefCell<FxHashMap<ThreadId, ThreadResult>>>,
    events: Rc<RefCell<FxHashMap<ThreadId, Rc<Event>>>>,
}

impl ThreadResultMap {
    pub fn new() -> Self {
        Self {
            tracked: Rc::new(RefCell::new(FxHashSet::default())),
            results: Rc::new(RefCell::new(FxHashMap::default())),
            events: Rc::new(RefCell::new(FxHashMap::default())),
        }
    }

    #[inline(always)]
    pub fn track(&self, id: ThreadId) {
        self.tracked.borrow_mut().insert(id);
    }

    #[inline(always)]
    pub fn is_tracked(&self, id: ThreadId) -> bool {
        self.tracked.borrow().contains(&id)
    }

    pub fn insert(&self, id: ThreadId, result: ThreadResult) {
        debug_assert!(self.is_tracked(id), "Thread must be tracked");
        self.results.borrow_mut().insert(id, result);
        if let Some(event) = self.events.borrow_mut().remove(&id) {
            event.notify(usize::MAX);
        }
    }

    pub async fn listen(&self, id: ThreadId) {
        debug_assert!(self.is_tracked(id), "Thread must be tracked");
        if !self.results.borrow().contains_key(&id) {
            let listener = {
                let mut events = self.events.borrow_mut();
                let event = events.entry(id).or_insert_with(|| Rc::new(Event::new()));
                event.listen()
            };
            listener.await;
        }
    }

    pub fn remove(&self, id: ThreadId) -> Option<ThreadResult> {
        let res = self.results.borrow_mut().remove(&id)?;
        self.tracked.borrow_mut().remove(&id);
        self.events.borrow_mut().remove(&id);
        Some(res)
    }
}
