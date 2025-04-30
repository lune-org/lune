use std::rc::Rc;

use concurrent_queue::ConcurrentQueue;
use mlua::prelude::*;

use crate::{threads::ThreadId, traits::IntoLuaThread};

use super::event::QueueEvent;

#[derive(Debug)]
struct ThreadQueueInner {
    queue: ConcurrentQueue<(LuaThread, LuaMultiValue)>,
    event: QueueEvent,
}

impl ThreadQueueInner {
    fn new() -> Self {
        let queue = ConcurrentQueue::unbounded();
        let event = QueueEvent::new();
        Self { queue, event }
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

        let _ = self.inner.queue.push((thread, args));
        self.inner.event.notify();

        Ok(id)
    }

    #[inline]
    pub fn drain_items(&self) -> impl Iterator<Item = (LuaThread, LuaMultiValue)> + '_ {
        self.inner.queue.try_iter()
    }

    #[inline]
    pub async fn wait_for_item(&self) {
        if self.inner.queue.is_empty() {
            let listener = self.inner.event.listen();
            // NOTE: Need to check again, we could have gotten
            // new queued items while creating our listener
            if self.inner.queue.is_empty() {
                listener.await;
            }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.queue.is_empty()
    }
}
