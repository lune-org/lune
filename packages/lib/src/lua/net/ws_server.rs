use std::sync::Arc;

use mlua::prelude::*;

use hyper::upgrade::Upgraded;
use hyper_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use crate::utils::table::TableBuilder;

#[derive(Debug, Clone)]
pub struct NetWebSocketServer(Arc<Mutex<WebSocketStream<Upgraded>>>);

impl NetWebSocketServer {
    pub async fn close(&self) -> LuaResult<()> {
        let mut ws = self.0.lock().await;
        ws.close(None).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn send(&self, msg: WsMessage) -> LuaResult<()> {
        let mut ws = self.0.lock().await;
        ws.send(msg).await.map_err(LuaError::external)?;
        Ok(())
    }

    pub async fn next(&self) -> LuaResult<Option<WsMessage>> {
        let mut ws = self.0.lock().await;
        let item = ws.next().await.transpose();
        item.map_err(LuaError::external)
    }

    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        // FIXME: Deallocate when closed
        let server = Box::leak(Box::new(self));
        TableBuilder::new(lua)?
            .with_async_function("close", |_, ()| async {
                let result = server.close().await;
                result
            })?
            .with_async_function("send", |_, message: String| async {
                let result = server.send(WsMessage::Text(message)).await;
                result
            })?
            .with_async_function("next", |lua, ()| async {
                let result = server.next().await?;
                Ok(match result {
                    Some(WsMessage::Binary(bin)) => LuaValue::String(lua.create_string(&bin)?),
                    Some(WsMessage::Text(txt)) => LuaValue::String(lua.create_string(&txt)?),
                    _ => LuaValue::Nil,
                })
            })?
            .build_readonly()
    }
}

impl From<WebSocketStream<Upgraded>> for NetWebSocketServer {
    fn from(value: WebSocketStream<Upgraded>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
