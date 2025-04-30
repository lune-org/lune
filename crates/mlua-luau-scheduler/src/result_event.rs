use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/**
    State which is highly optimized for a single notification event.

    `Some` means not notified yet, `None` means notified.
*/
#[derive(Debug, Default)]
struct OnceEventState {
    wakers: RefCell<Option<Vec<Waker>>>,
}

impl OnceEventState {
    fn new() -> Self {
        Self {
            wakers: RefCell::new(Some(Vec::new())),
        }
    }
}

/**
    An event that may be notified exactly once.

    May be cheaply cloned.
*/
#[derive(Debug, Clone, Default)]
pub struct OnceEvent {
    state: Rc<OnceEventState>,
}

impl OnceEvent {
    /**
        Creates a new event that can be notified exactly once.
    */
    pub fn new() -> Self {
        let initial_state = OnceEventState::new();
        Self {
            state: Rc::new(initial_state),
        }
    }

    /**
        Notifies waiting listeners.

        This is idempotent; subsequent calls do nothing.
    */
    pub fn notify(&self) {
        if let Some(wakers) = { self.state.wakers.borrow_mut().take() } {
            for waker in wakers {
                waker.wake();
            }
        }
    }

    /**
        Creates a listener that implements `Future` and resolves when `notify` is called.

        If `notify` has already been called, the future will resolve immediately.
    */
    pub fn listen(&self) -> OnceListener {
        OnceListener {
            state: self.state.clone(),
        }
    }
}

/**
    A listener that resolves when the event is notified.

    May be cheaply cloned.

    See [`OnceEvent`] for more information.
*/
#[derive(Debug)]
pub struct OnceListener {
    state: Rc<OnceEventState>,
}

impl Future for OnceListener {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut wakers_guard = self.state.wakers.borrow_mut();
        match &mut *wakers_guard {
            Some(wakers) => {
                // Not yet notified
                if !wakers.iter().any(|w| w.will_wake(cx.waker())) {
                    wakers.push(cx.waker().clone());
                }
                Poll::Pending
            }
            None => {
                // Already notified
                Poll::Ready(())
            }
        }
    }
}

impl Unpin for OnceListener {}
