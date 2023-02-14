use std::sync::Arc;

use mlua::prelude::*;

use hyper::upgrade::Upgraded;
use hyper_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use crate::utils::table::TableBuilder;

type Inner = Arc<Mutex<WebSocketStream<Upgraded>>>;

#[derive(Debug, Clone)]
pub struct NetWebSocketServer(Inner);

impl NetWebSocketServer {
    async fn close(&self) -> LuaResult<()> {
        self.0.lock().await.close(None).await;
        Ok(())
    }

    async fn send(&self, msg: String) -> LuaResult<()> {
        self.0
            .lock()
            .await
            .send(WsMessage::Text(msg))
            .await
            .map_err(LuaError::external)
    }

    async fn next<'a>(&self, lua: &'static Lua) -> LuaResult<LuaValue<'a>> {
        let item = self
            .0
            .lock()
            .await
            .next()
            .await
            .transpose()
            .map_err(LuaError::external)?;
        Ok(match item {
            None => LuaValue::Nil,
            Some(msg) => match msg {
                WsMessage::Binary(bin) => LuaValue::String(lua.create_string(&bin)?),
                WsMessage::Text(txt) => LuaValue::String(lua.create_string(&txt)?),
                _ => LuaValue::Nil,
            },
        })
    }

    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        let inner_close = self.clone();
        let inner_send = self.clone();
        let inner_next = self.clone();
        TableBuilder::new(lua)?
            .with_async_function("close", move |_, _: ()| inner_close.close())?
            .with_async_function("send", move |_, msg: String| inner_send.send(msg))?
            .with_async_function("next", move |lua, _: ()| inner_next.next(lua))?
            .build_readonly()
    }
}

impl From<WebSocketStream<Upgraded>> for NetWebSocketServer {
    fn from(value: WebSocketStream<Upgraded>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
