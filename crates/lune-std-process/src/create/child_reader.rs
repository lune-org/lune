use std::sync::Arc;

use async_lock::Mutex as AsyncMutex;
use async_process::{ChildStderr as AsyncChildStderr, ChildStdout as AsyncChildStdout};
use futures_lite::prelude::*;

use mlua::prelude::*;

const DEFAULT_BUFFER_SIZE: usize = 1024;

// Inner (plumbing) implementation

#[derive(Debug)]
enum ChildReaderInner {
    None,
    Stdout(AsyncChildStdout),
    Stderr(AsyncChildStderr),
}

impl ChildReaderInner {
    async fn read(&mut self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        if matches!(self, ChildReaderInner::None) {
            return Ok(Vec::new());
        }

        let mut buf = vec![0; size];

        let read = match self {
            ChildReaderInner::None => unreachable!(),
            ChildReaderInner::Stdout(stdout) => stdout.read(&mut buf).await?,
            ChildReaderInner::Stderr(stderr) => stderr.read(&mut buf).await?,
        };

        buf.truncate(read);

        Ok(buf)
    }

    async fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = Vec::new();

        let read = match self {
            ChildReaderInner::None => 0,
            ChildReaderInner::Stdout(stdout) => stdout.read_to_end(&mut buf).await?,
            ChildReaderInner::Stderr(stderr) => stderr.read_to_end(&mut buf).await?,
        };

        buf.truncate(read);

        Ok(buf)
    }
}

impl From<AsyncChildStdout> for ChildReaderInner {
    fn from(stdout: AsyncChildStdout) -> Self {
        Self::Stdout(stdout)
    }
}

impl From<AsyncChildStderr> for ChildReaderInner {
    fn from(stderr: AsyncChildStderr) -> Self {
        Self::Stderr(stderr)
    }
}

impl From<Option<AsyncChildStdout>> for ChildReaderInner {
    fn from(stdout: Option<AsyncChildStdout>) -> Self {
        stdout.map_or(Self::None, Into::into)
    }
}

impl From<Option<AsyncChildStderr>> for ChildReaderInner {
    fn from(stderr: Option<AsyncChildStderr>) -> Self {
        stderr.map_or(Self::None, Into::into)
    }
}

// Outer (lua-accessible, clonable) implementation

#[derive(Debug, Clone)]
pub struct ChildReader {
    inner: Arc<AsyncMutex<ChildReaderInner>>,
}

impl LuaUserData for ChildReader {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("read", |lua, this, size: Option<usize>| {
            let inner = this.inner.clone();
            let size = size.unwrap_or(DEFAULT_BUFFER_SIZE);
            async move {
                let mut inner = inner.lock().await;
                let bytes = inner.read(size).await.into_lua_err()?;
                if bytes.is_empty() {
                    Ok(LuaValue::Nil)
                } else {
                    Ok(LuaValue::String(lua.create_string(bytes)?))
                }
            }
        });
        methods.add_async_method("readToEnd", |lua, this, (): ()| {
            let inner = this.inner.clone();
            async move {
                let mut inner = inner.lock().await;
                let bytes = inner.read_to_end().await.into_lua_err()?;
                Ok(lua.create_string(bytes))
            }
        });
    }
}

impl<T: Into<ChildReaderInner>> From<T> for ChildReader {
    fn from(inner: T) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(inner.into())),
        }
    }
}
