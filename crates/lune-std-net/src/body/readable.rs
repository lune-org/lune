use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::body::{Body, Bytes, Frame, SizeHint};
use mlua::prelude::*;

use super::cursor::ReadableBodyCursor;

/**
    Zero-copy wrapper for a readable body.

    Provides methods to read bytes that can be safely used if, and only
    if, the respective Lua struct for the body has not yet been dropped.

    If the body was created from a `Vec<u8>`, `Bytes`, or a `String`, reading
    bytes is always safe and does not go through any additional indirections.
*/
#[derive(Debug, Clone)]
pub struct ReadableBody {
    cursor: Option<ReadableBodyCursor>,
}

impl ReadableBody {
    pub const fn empty() -> Self {
        Self { cursor: None }
    }

    pub fn as_slice(&self) -> &[u8] {
        match self.cursor.as_ref() {
            Some(cursor) => cursor.as_slice(),
            None => &[],
        }
    }

    pub fn into_bytes(self) -> Bytes {
        match self.cursor {
            Some(cursor) => cursor.into_bytes(),
            None => Bytes::new(),
        }
    }
}

impl Body for ReadableBody {
    type Data = ReadableBodyCursor;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.cursor.take().map(|d| Ok(Frame::data(d))))
    }

    fn is_end_stream(&self) -> bool {
        self.cursor.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        self.cursor.as_ref().map_or_else(
            || SizeHint::with_exact(0),
            |c| SizeHint::with_exact(c.len() as u64),
        )
    }
}

impl<T> From<T> for ReadableBody
where
    T: Into<ReadableBodyCursor>,
{
    fn from(value: T) -> Self {
        Self {
            cursor: Some(value.into()),
        }
    }
}

impl<T> From<Option<T>> for ReadableBody
where
    T: Into<ReadableBodyCursor>,
{
    fn from(value: Option<T>) -> Self {
        Self {
            cursor: value.map(Into::into),
        }
    }
}

impl FromLua for ReadableBody {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(Self::empty()),
            LuaValue::String(str) => Ok(Self::from(str)),
            LuaValue::Buffer(buf) => Ok(Self::from(buf)),
            v => Err(LuaError::FromLuaConversionError {
                from: v.type_name(),
                to: "Body".to_string(),
                message: Some(format!(
                    "Invalid body - expected string or buffer, got {}",
                    v.type_name()
                )),
            }),
        }
    }
}
