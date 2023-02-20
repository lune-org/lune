use std::sync::Arc;

use hyper::upgrade::Upgraded;
use mlua::prelude::*;

use hyper_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::Mutex,
};
use tokio_tungstenite::MaybeTlsStream;

use crate::utils::table::TableBuilder;

#[derive(Debug, Clone)]
pub struct NetWebSocket<T> {
    stream: Arc<Mutex<WebSocketStream<T>>>,
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(value: WebSocketStream<T>) -> Self {
        Self {
            stream: Arc::new(Mutex::new(value)),
        }
    }

    pub async fn close(&self) -> LuaResult<()> {
        let mut inner = self.stream.lock().await;
        inner.close(None).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn send(&self, msg: WsMessage) -> LuaResult<()> {
        let mut inner = self.stream.lock().await;
        inner.send(msg).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn next(&self) -> LuaResult<Option<WsMessage>> {
        let mut inner = self.stream.lock().await;
        let item = inner.next().await.transpose();
        item.map_err(LuaError::external)
    }

    pub async fn send_string(&self, msg: String) -> LuaResult<()> {
        self.send(WsMessage::Text(msg)).await
    }

    pub async fn next_lua_value(&self, lua: &'static Lua) -> LuaResult<LuaValue> {
        Ok(match self.next().await? {
            Some(WsMessage::Binary(bin)) => LuaValue::String(lua.create_string(&bin)?),
            Some(WsMessage::Text(txt)) => LuaValue::String(lua.create_string(&txt)?),
            _ => LuaValue::Nil,
        })
    }
}

impl NetWebSocket<MaybeTlsStream<TcpStream>> {
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        // FIXME: Deallocate when closed
        let sock = Box::leak(Box::new(self));
        TableBuilder::new(lua)?
            .with_async_function("close", |_, ()| sock.close())?
            .with_async_function("send", |_, msg: String| sock.send_string(msg))?
            .with_async_function("next", |lua, ()| sock.next_lua_value(lua))?
            .build_readonly()
    }
}

impl NetWebSocket<Upgraded> {
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        // FIXME: Deallocate when closed
        let sock = Box::leak(Box::new(self));
        TableBuilder::new(lua)?
            .with_async_function("close", |_, ()| sock.close())?
            .with_async_function("send", |_, msg: String| sock.send_string(msg))?
            .with_async_function("next", |lua, ()| sock.next_lua_value(lua))?
            .build_readonly()
    }
}
