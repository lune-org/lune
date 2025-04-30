use std::ops::{Deref, DerefMut};

use super::threads::ThreadQueue;

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
