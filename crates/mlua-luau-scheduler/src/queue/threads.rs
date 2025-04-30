#![allow(clippy::inline_always)]

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use mlua::prelude::*;

use crate::{threads::ThreadId, traits::IntoLuaThread};

use super::event::QueueEvent;

#[derive(Debug)]
struct ThreadQueueInner {
    queue: RefCell<VecDeque<(LuaThread, LuaMultiValue)>>,
    event: QueueEvent,
}

impl ThreadQueueInner {
    fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            event: QueueEvent::new(),
        }
    }
}

/**
    Queue for storing [`LuaThread`]s with associated arguments.

    Provides methods for pushing and draining the queue, as
    well as listening for new items being pushed to the queue.
*/
#[derive(Debug, Clone)]
pub(crate) struct ThreadQueue {
    inner: Rc<ThreadQueueInner>,
}

impl ThreadQueue {
    pub fn new() -> Self {
        let inner = Rc::new(ThreadQueueInner::new());
        Self { inner }
    }

    pub fn push_item(
        &self,
        lua: &Lua,
        thread: impl IntoLuaThread,
        args: impl IntoLuaMulti,
    ) -> LuaResult<ThreadId> {
        let thread = thread.into_lua_thread(lua)?;
        let args = args.into_lua_multi(lua)?;

        tracing::trace!("pushing item to queue with {} args", args.len());
        let id = ThreadId::from(&thread);

        self.inner.queue.borrow_mut().push_back((thread, args));
        self.inner.event.notify();

        Ok(id)
    }

    #[inline(always)]
    pub fn drain_items(&self) -> ThreadQueueDrain<'_> {
        ThreadQueueDrain::new(self)
    }

    #[inline(always)]
    pub async fn wait_for_item(&self) {
        if self.inner.queue.borrow().is_empty() {
            self.inner.event.listen().await;
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.queue.borrow().is_empty()
    }
}

/**
    Iterator that drains the thread queue,
    popping items from the front first.
*/
pub(crate) struct ThreadQueueDrain<'a> {
    queue: &'a ThreadQueue,
}

impl<'a> ThreadQueueDrain<'a> {
    pub fn new(queue: &'a ThreadQueue) -> Self {
        Self { queue }
    }
}

impl Iterator for ThreadQueueDrain<'_> {
    type Item = (LuaThread, LuaMultiValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.inner.queue.borrow_mut().pop_front()
    }
}
