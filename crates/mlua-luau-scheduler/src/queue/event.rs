use std::{
    cell::{Cell, RefCell},
    future::Future,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/**
    Internal state for queue events.
*/
#[derive(Debug, Default)]
struct QueueEventState {
    generation: Cell<u64>,
    wakers: RefCell<Vec<Waker>>,
}

/**
    A single-threaded event signal that can be notified multiple times.
*/
#[derive(Debug, Clone, Default)]
pub(crate) struct QueueEvent {
    state: Rc<QueueEventState>,
}

impl QueueEvent {
    /**
        Creates a new event.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Notifies all waiting listeners.
    */
    pub fn notify(&self) {
        self.state.generation.set(self.state.generation.get() + 1);

        let wakers = {
            let mut wakers = self.state.wakers.borrow_mut();
            mem::take(&mut *wakers)
        };

        for waker in wakers {
            waker.wake();
        }
    }

    /**
        Creates a listener that implements `Future` and resolves when `notify` is called.
    */
    pub fn listen(&self) -> QueueListener {
        QueueListener {
            state: self.state.clone(),
            generation: self.state.generation.get(),
        }
    }
}

/**
    A listener future that resolves when the corresponding [`QueueEvent`] is notified.
*/
#[derive(Debug)]
pub(crate) struct QueueListener {
    state: Rc<QueueEventState>,
    generation: u64,
}

impl Future for QueueListener {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if notify was called (generation is more recent)
        let current = self.state.generation.get();
        if current > self.generation {
            self.get_mut().generation = current;
            return Poll::Ready(());
        }

        // No notification observed yet
        let mut wakers = self.state.wakers.borrow_mut();
        if !wakers.iter().any(|w| w.will_wake(cx.waker())) {
            wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }
}

impl Unpin for QueueListener {}
