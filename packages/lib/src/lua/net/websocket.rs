use std::sync::Arc;

use mlua::prelude::*;

use hyper_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};

use crate::lua::table::TableBuilder;

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
        let mut ws = self.stream.lock().await;
        ws.close(None).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn send(&self, msg: WsMessage) -> LuaResult<()> {
        let mut ws = self.stream.lock().await;
        ws.send(msg).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn next(&self) -> LuaResult<Option<WsMessage>> {
        let mut ws = self.stream.lock().await;
        let item = ws.next().await.transpose();
        item.map_err(LuaError::external)
    }
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin + 'static,
{
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        let ws = Box::leak(Box::new(self));
        TableBuilder::new(lua)?
            .with_async_function("close", |_, _: ()| async { ws.close().await })?
            .with_async_function("send", |_, msg: String| async {
                ws.send(WsMessage::Text(msg)).await
            })?
            .with_async_function("next", |_, _: ()| async {
                match ws.next().await? {
                    Some(msg) => Ok(match msg {
                        WsMessage::Binary(bin) => LuaValue::String(lua.create_string(&bin)?),
                        WsMessage::Text(txt) => LuaValue::String(lua.create_string(&txt)?),
                        _ => LuaValue::Nil,
                    }),
                    None => Ok(LuaValue::Nil),
                }
            })?
            .build_readonly()
    }
}
