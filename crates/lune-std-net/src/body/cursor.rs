use hyper::body::{Buf, Bytes};

use super::inner::ReadableBodyInner;

/**
    The cursor keeping track of inner data and its position for a readable body.
*/
#[derive(Debug, Clone)]
pub struct ReadableBodyCursor {
    inner: ReadableBodyInner,
    start: usize,
}

impl ReadableBodyCursor {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.inner.as_slice()[self.start..]
    }

    pub fn advance(&mut self, cnt: usize) {
        self.start += cnt;
        if self.start > self.inner.len() {
            self.start = self.inner.len();
        }
    }

    pub fn into_bytes(self) -> Bytes {
        self.inner.into_bytes()
    }
}

impl Buf for ReadableBodyCursor {
    fn remaining(&self) -> usize {
        self.len().saturating_sub(self.start)
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        self.advance(cnt);
    }
}

impl<T> From<T> for ReadableBodyCursor
where
    T: Into<ReadableBodyInner>,
{
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
            start: 0,
        }
    }
}
