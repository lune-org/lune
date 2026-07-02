#![allow(clippy::inline_always)]

use std::{cell::RefCell, mem, rc::Rc};

use mlua::prelude::*;

use crate::{threads::ThreadId, traits::IntoLuaThread};

use crate::events::MultiEvent;

#[derive(Debug)]
struct ThreadQueueInner {
    queue: RefCell<Vec<(LuaThread, LuaMultiValue)>>,
    event: MultiEvent,
}

impl ThreadQueueInner {
    fn new() -> Self {
        Self {
            queue: RefCell::new(Vec::new()),
            event: MultiEvent::new(),
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

        self.inner.queue.borrow_mut().push((thread, args));
        self.inner.event.notify();

        Ok(id)
    }

    #[inline(always)]
    pub fn take_items(&self) -> Vec<(LuaThread, LuaMultiValue)> {
        let mut queue = self.inner.queue.borrow_mut();
        mem::take(&mut *queue)
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
