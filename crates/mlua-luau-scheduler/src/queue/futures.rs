use std::{pin::Pin, rc::Rc};

use concurrent_queue::ConcurrentQueue;
use event_listener::Event;
use futures_lite::{Future, FutureExt};

pub type LocalBoxFuture<'fut> = Pin<Box<dyn Future<Output = ()> + 'fut>>;

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
