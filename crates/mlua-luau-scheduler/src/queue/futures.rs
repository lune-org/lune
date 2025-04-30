use std::{cell::RefCell, mem, pin::Pin, rc::Rc};

use futures_lite::prelude::*;

use super::event::QueueEvent;

pub type LocalBoxFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

struct FuturesQueueInner<'fut> {
    queue: RefCell<Vec<LocalBoxFuture<'fut>>>,
    event: QueueEvent,
}

impl FuturesQueueInner<'_> {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(Vec::new()),
            event: QueueEvent::new(),
        }
    }
}

/**
    Queue for storing local futures.

    Provides methods for pushing and draining the queue, as
    well as listening for new items being pushed to the queue.
*/
#[derive(Clone)]
pub(crate) struct FuturesQueue<'fut> {
    inner: Rc<FuturesQueueInner<'fut>>,
}

impl<'fut> FuturesQueue<'fut> {
    pub fn new() -> Self {
        let inner = Rc::new(FuturesQueueInner::new());
        Self { inner }
    }

    pub fn push_item(&self, fut: impl Future<Output = ()> + 'fut) {
        self.inner.queue.borrow_mut().push(fut.boxed_local());
        self.inner.event.notify();
    }

    pub fn take_items(&self) -> Vec<LocalBoxFuture<'fut>> {
        let mut queue = self.inner.queue.borrow_mut();
        mem::take(&mut *queue)
    }

    pub async fn wait_for_item(&self) {
        if self.inner.queue.borrow().is_empty() {
            self.inner.event.listen().await;
        }
    }
}
