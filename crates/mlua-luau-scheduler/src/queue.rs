use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
};

use concurrent_queue::ConcurrentQueue;
use event_listener::Event;
use futures_lite::{Future, FutureExt};
use mlua::prelude::*;

use crate::{traits::IntoLuaThread, ThreadId};

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
        self.inner.event.notify(usize::MAX);

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

/**
    Alias for [`ThreadQueue`], providing a newtype to store in Lua app data.
*/
#[derive(Debug, Clone)]
pub(crate) struct SpawnedThreadQueue(ThreadQueue);

impl SpawnedThreadQueue {
    pub fn new() -> Self {
        Self(ThreadQueue::new())
    }
}

impl Deref for SpawnedThreadQueue {
    type Target = ThreadQueue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpawnedThreadQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/**
    Alias for [`ThreadQueue`], providing a newtype to store in Lua app data.
*/
#[derive(Debug, Clone)]
pub(crate) struct DeferredThreadQueue(ThreadQueue);

impl DeferredThreadQueue {
    pub fn new() -> Self {
        Self(ThreadQueue::new())
    }
}

impl Deref for DeferredThreadQueue {
    type Target = ThreadQueue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DeferredThreadQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type LocalBoxFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

/**
    Queue for storing local futures.

    Provides methods for pushing and draining the queue, as
    well as listening for new items being pushed to the queue.
*/
#[derive(Debug, Clone)]
pub(crate) struct FuturesQueue<'fut> {
    inner: Rc<FuturesQueueInner<'fut>>,
}

impl<'fut> FuturesQueue<'fut> {
    pub fn new() -> Self {
        let inner = Rc::new(FuturesQueueInner::new());
        Self { inner }
    }

    pub fn push_item(&self, fut: impl Future<Output = ()> + 'fut) {
        let _ = self.inner.queue.push(fut.boxed_local());
        self.inner.event.notify(usize::MAX);
    }

    pub fn drain_items<'outer>(
        &'outer self,
    ) -> impl Iterator<Item = LocalBoxFuture<'fut>> + 'outer {
        self.inner.queue.try_iter()
    }

    pub async fn wait_for_item(&self) {
        if self.inner.queue.is_empty() {
            self.inner.event.listen().await;
        }
    }
}

// Inner structs without ref counting so that outer structs
// have only a single ref counter for extremely cheap clones

#[derive(Debug)]
struct ThreadQueueInner {
    queue: ConcurrentQueue<(LuaThread, LuaMultiValue)>,
    event: Event,
}

impl ThreadQueueInner {
    fn new() -> Self {
        let queue = ConcurrentQueue::unbounded();
        let event = Event::new();
        Self { queue, event }
    }
}

#[derive(Debug)]
struct FuturesQueueInner<'fut> {
    queue: ConcurrentQueue<LocalBoxFuture<'fut>>,
    event: Event,
}

impl FuturesQueueInner<'_> {
    pub fn new() -> Self {
        let queue = ConcurrentQueue::unbounded();
        let event = Event::new();
        Self { queue, event }
    }
}
