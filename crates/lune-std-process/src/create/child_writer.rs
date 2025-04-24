use std::sync::Arc;

use async_lock::Mutex as AsyncMutex;
use async_process::ChildStdin as AsyncChildStdin;
use futures_lite::prelude::*;

use bstr::BString;
use mlua::prelude::*;

// Inner (plumbing) implementation

#[derive(Debug)]
enum ChildWriterInner {
    None,
    Stdin(AsyncChildStdin),
}

impl ChildWriterInner {
    async fn write(&mut self, data: Vec<u8>) -> Result<(), std::io::Error> {
        match self {
            ChildWriterInner::None => Ok(()),
            ChildWriterInner::Stdin(stdin) => stdin.write_all(&data).await,
        }
    }

    async fn close(&mut self) -> Result<(), std::io::Error> {
        match self {
            ChildWriterInner::None => Ok(()),
            ChildWriterInner::Stdin(stdin) => stdin.close().await,
        }
    }
}

impl From<AsyncChildStdin> for ChildWriterInner {
    fn from(stdin: AsyncChildStdin) -> Self {
        ChildWriterInner::Stdin(stdin)
    }
}

impl From<Option<AsyncChildStdin>> for ChildWriterInner {
    fn from(stdin: Option<AsyncChildStdin>) -> Self {
        stdin.map_or(Self::None, Into::into)
    }
}

// Outer (lua-accessible, clonable) implementation

#[derive(Debug, Clone)]
pub struct ChildWriter {
    inner: Arc<AsyncMutex<ChildWriterInner>>,
}

impl LuaUserData for ChildWriter {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("write", |_, this, data: BString| {
            let inner = this.inner.clone();
            let data = data.to_vec();
            async move {
                let mut inner = inner.lock().await;
                inner.write(data).await.into_lua_err()
            }
        });
        methods.add_async_method("close", |_, this, (): ()| {
            let inner = this.inner.clone();
            async move {
                let mut inner = inner.lock().await;
                inner.close().await.into_lua_err()
            }
        });
    }
}

impl<T: Into<ChildWriterInner>> From<T> for ChildWriter {
    fn from(inner: T) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(inner.into())),
        }
    }
}
