use hyper::body::{Buf as _, Bytes};
use mlua::{prelude::*, Buffer as LuaBuffer};

/**
    The inner data for a readable body.
*/
#[derive(Debug, Clone)]
pub enum ReadableBodyInner {
    Bytes(Bytes),
    String(String),
    LuaString(LuaString),
    LuaBuffer(LuaBuffer),
}

impl ReadableBodyInner {
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(b) => b.len(),
            Self::String(s) => s.len(),
            Self::LuaString(s) => s.as_bytes().len(),
            Self::LuaBuffer(b) => b.len(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        /*
            SAFETY: Reading lua strings and lua buffers as raw slices is safe while we can
            guarantee that the inner Lua value + main lua struct has not yet been dropped

            1. Buffers are fixed-size and guaranteed to never resize
            2. We do not expose any method for writing to the body, only reading
            3. We guarantee that net.request and net.serve futures are only driven forward
                while we also know that the Lua + scheduler pair have not yet been dropped
            4. Any writes from within lua to a buffer, are considered user error,
               and are not unsafe, since the only possible outcome with the above
               guarantees is invalid / mangled contents in request / response bodies
        */
        match self {
            Self::Bytes(b) => b.chunk(),
            Self::String(s) => s.as_bytes(),
            Self::LuaString(s) => unsafe {
                // BorrowedBytes would not let us return a plain slice here,
                // which is what the Buf implementation below needs - we need to
                // do a little hack here to re-create the slice without a lifetime
                let b = s.as_bytes();

                let ptr = b.as_ptr();
                let len = b.len();

                std::slice::from_raw_parts(ptr, len)
            },
            Self::LuaBuffer(b) => unsafe {
                // Similar to above, we need to get the raw slice for the buffer,
                // which is a bit trickier here because Buffer has a read + write
                // interface instead of using slices for some unknown reason
                let v = LuaValue::Buffer(b.clone());

                let ptr = v.to_pointer().cast::<u8>();
                let len = b.len();

                std::slice::from_raw_parts(ptr, len)
            },
        }
    }

    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Bytes(b) => b,
            Self::String(s) => Bytes::from(s),
            Self::LuaString(s) => Bytes::from(s.as_bytes().to_vec()),
            Self::LuaBuffer(b) => Bytes::from(b.to_vec()),
        }
    }
}

impl From<&'static str> for ReadableBodyInner {
    fn from(value: &'static str) -> Self {
        Self::Bytes(Bytes::from(value))
    }
}

impl From<Vec<u8>> for ReadableBodyInner {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Bytes::from(value))
    }
}

impl From<Bytes> for ReadableBodyInner {
    fn from(value: Bytes) -> Self {
        Self::Bytes(value)
    }
}

impl From<String> for ReadableBodyInner {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<LuaString> for ReadableBodyInner {
    fn from(value: LuaString) -> Self {
        Self::LuaString(value)
    }
}

impl From<LuaBuffer> for ReadableBodyInner {
    fn from(value: LuaBuffer) -> Self {
        Self::LuaBuffer(value)
    }
}
