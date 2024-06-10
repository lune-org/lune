use std::{pin::Pin, rc::Rc};

use concurrent_queue::ConcurrentQueue;
use derive_more::{Deref, DerefMut};
use event_listener::Event;
use futures_lite::{Future, FutureExt};
use mlua::prelude::*;

use crate::{traits::IntoLuaThread, util::ThreadWithArgs, ThreadId};

/**
    Queue for storing [`LuaThread`]s with associated arguments.

    Provides methods for pushing and draining the queue, as
    well as listening for new items being pushed to the queue.
*/
#[derive(Debug, Clone)]
pub(crate) struct ThreadQueue {
    queue: Rc<ConcurrentQueue<ThreadWithArgs>>,
    event: Rc<Event>,
}

impl ThreadQueue {
    pub fn new() -> Self {
        let queue = Rc::new(ConcurrentQueue::unbounded());
        let event = Rc::new(Event::new());
        Self { queue, event }
    }

    pub fn push_item<'lua>(
        &self,
        lua: &'lua Lua,
        thread: impl IntoLuaThread<'lua>,
        args: impl IntoLuaMulti<'lua>,
    ) -> LuaResult<ThreadId> {
        let thread = thread.into_lua_thread(lua)?;
        let args = args.into_lua_multi(lua)?;

        tracing::trace!("pushing item to queue with {} args", args.len());
        let id = ThreadId::from(&thread);
        let stored = ThreadWithArgs::new(lua, thread, args)?;

        self.queue.push(stored).into_lua_err()?;
        self.event.notify(usize::MAX);

        Ok(id)
    }

    #[inline]
    pub fn drain_items<'outer, 'lua>(
        &'outer self,
        lua: &'lua Lua,
    ) -> impl Iterator<Item = (LuaThread<'lua>, LuaMultiValue<'lua>)> + 'outer
    where
        'lua: 'outer,
    {
        self.queue.try_iter().map(|stored| stored.into_inner(lua))
    }

    #[inline]
    pub async fn wait_for_item(&self) {
        if self.queue.is_empty() {
            let listener = self.event.listen();
            // NOTE: Need to check again, we could have gotten
            // new queued items while creating our listener
            if self.queue.is_empty() {
                listener.await;
            }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/**
    Alias for [`ThreadQueue`], providing a newtype to store in Lua app data.
*/
#[derive(Debug, Clone, Deref, DerefMut)]
pub(crate) struct SpawnedThreadQueue(ThreadQueue);

impl SpawnedThreadQueue {
    pub fn new() -> Self {
        Self(ThreadQueue::new())
    }
}

/**
    Alias for [`ThreadQueue`], providing a newtype to store in Lua app data.
*/
#[derive(Debug, Clone, Deref, DerefMut)]
pub(crate) struct DeferredThreadQueue(ThreadQueue);

impl DeferredThreadQueue {
    pub fn new() -> Self {
        Self(ThreadQueue::new())
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
    queue: Rc<ConcurrentQueue<LocalBoxFuture<'fut>>>,
    event: Rc<Event>,
}

impl<'fut> FuturesQueue<'fut> {
    pub fn new() -> Self {
        let queue = Rc::new(ConcurrentQueue::unbounded());
        let event = Rc::new(Event::new());
        Self { queue, event }
    }

    pub fn push_item(&self, fut: impl Future<Output = ()> + 'fut) {
        let _ = self.queue.push(fut.boxed_local());
        self.event.notify(usize::MAX);
    }

    pub fn drain_items<'outer>(
        &'outer self,
    ) -> impl Iterator<Item = LocalBoxFuture<'fut>> + 'outer {
        self.queue.try_iter()
    }

    pub async fn wait_for_item(&self) {
        if self.queue.is_empty() {
            self.event.listen().await;
        }
    }
}
