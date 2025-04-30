use std::ops::{Deref, DerefMut};

use super::threads::ThreadQueue;

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
