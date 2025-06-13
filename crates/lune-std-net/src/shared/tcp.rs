use std::{io::Error, net::SocketAddr, sync::Arc};

use async_lock::Mutex as AsyncMutex;
use bstr::BString;
use futures::{
    io::{ReadHalf, WriteHalf},
    prelude::*,
};

use mlua::prelude::*;

use crate::client::stream::MaybeTlsStream;

const DEFAULT_BUFFER_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct Tcp {
    local_addr: Arc<Option<SocketAddr>>,
    remote_addr: Arc<Option<SocketAddr>>,
    read_half: Arc<AsyncMutex<ReadHalf<MaybeTlsStream>>>,
    write_half: Arc<AsyncMutex<WriteHalf<MaybeTlsStream>>>,
}

impl Tcp {
    async fn read(&self, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0; size];

        let mut handle = self.read_half.lock().await;
        let read = handle.read(&mut buf).await?;

        buf.truncate(read);

        Ok(buf)
    }

    async fn write(&self, data: Vec<u8>) -> Result<(), Error> {
        let mut handle = self.write_half.lock().await;
        handle.write_all(&data).await?;

        Ok(())
    }

    async fn close(&self) -> Result<(), Error> {
        let mut handle = self.write_half.lock().await;

        handle.close().await?;

        Ok(())
    }
}

impl<T> From<T> for Tcp
where
    T: Into<MaybeTlsStream>,
{
    fn from(value: T) -> Self {
        let stream = value.into();

        let local_addr = stream.local_addr().ok();
        let remote_addr = stream.remote_addr().ok();

        let (read, write) = stream.split();

        Self {
            local_addr: Arc::new(local_addr),
            remote_addr: Arc::new(remote_addr),
            read_half: Arc::new(AsyncMutex::new(read)),
            write_half: Arc::new(AsyncMutex::new(write)),
        }
    }
}

impl LuaUserData for Tcp {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("localIp", |_, this| {
            Ok(this.local_addr.map(|address| address.ip().to_string()))
        });
        fields.add_field_method_get("localPort", |_, this| {
            Ok(this.local_addr.map(|address| address.port()))
        });
        fields.add_field_method_get("remoteIp", |_, this| {
            Ok(this.remote_addr.map(|address| address.ip().to_string()))
        });
        fields.add_field_method_get("remotePort", |_, this| {
            Ok(this.remote_addr.map(|address| address.port()))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("read", |_, this, size: Option<usize>| {
            let this = this.clone();
            let size = size.unwrap_or(DEFAULT_BUFFER_SIZE);
            async move { this.read(size).await.into_lua_err() }
        });
        methods.add_async_method("write", |_, this, data: BString| {
            let this = this.clone();
            let data = data.to_vec();
            async move { this.write(data).await.into_lua_err() }
        });
        methods.add_async_method("close", |_, this, (): ()| {
            let this = this.clone();
            async move { this.close().await.into_lua_err() }
        });
    }
}
