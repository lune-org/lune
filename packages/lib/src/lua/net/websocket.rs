use std::{cell::Cell, sync::Arc};

use mlua::prelude::*;

use hyper_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode as WsCloseCode, CloseFrame as WsCloseFrame},
        Message as WsMessage,
    },
    WebSocketStream,
};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};

use crate::lua::table::TableBuilder;

#[derive(Debug, Clone)]
pub struct NetWebSocket<T> {
    close_code: Cell<Option<u16>>,
    stream: Arc<Mutex<WebSocketStream<T>>>,
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(value: WebSocketStream<T>) -> Self {
        Self {
            close_code: Cell::new(None),
            stream: Arc::new(Mutex::new(value)),
        }
    }

    pub fn get_lua_close_code(&self) -> LuaValue {
        match self.close_code.get() {
            Some(code) => LuaValue::Number(code as f64),
            None => LuaValue::Nil,
        }
    }

    pub async fn close(&self, code: Option<u16>) -> LuaResult<()> {
        let mut ws = self.stream.lock().await;
        let res = ws.close(Some(WsCloseFrame {
            code: match code {
                Some(code) if (1000..=4999).contains(&code) => WsCloseCode::from(code),
                Some(code) => {
                    return Err(LuaError::RuntimeError(format!(
                        "Close code must be between 1000 and 4999, got {code}"
                    )))
                }
                None => WsCloseCode::Normal,
            },
            reason: "".into(),
        }));
        res.await.map_err(LuaError::external)
    }

    pub async fn send(&self, msg: WsMessage) -> LuaResult<()> {
        let mut ws = self.stream.lock().await;
        ws.send(msg).await.map_err(LuaError::external)
    }

    pub async fn send_lua_string<'lua>(
        &self,
        string: LuaString<'lua>,
        as_binary: Option<bool>,
    ) -> LuaResult<()> {
        let msg = if matches!(as_binary, Some(true)) {
            WsMessage::Binary(string.as_bytes().to_vec())
        } else {
            let s = string.to_str().map_err(LuaError::external)?;
            WsMessage::Text(s.to_string())
        };
        self.send(msg).await
    }

    pub async fn next(&self) -> LuaResult<Option<WsMessage>> {
        let mut ws = self.stream.lock().await;
        let item = ws.next().await.transpose().map_err(LuaError::external);
        match item {
            Ok(Some(WsMessage::Close(msg))) => {
                if let Some(msg) = &msg {
                    self.close_code.replace(Some(msg.code.into()));
                }
                Ok(Some(WsMessage::Close(msg)))
            }
            val => val,
        }
    }

    pub async fn next_lua_string<'lua>(&'lua self, lua: &'lua Lua) -> LuaResult<LuaValue> {
        while let Some(msg) = self.next().await? {
            let msg_string_opt = match msg {
                WsMessage::Binary(bin) => Some(lua.create_string(&bin)?),
                WsMessage::Text(txt) => Some(lua.create_string(&txt)?),
                // Stop waiting for next message if we get a close message
                WsMessage::Close(_) => return Ok(LuaValue::Nil),
                // Ignore ping/pong/frame messages, they are handled by tungstenite
                _ => None,
            };
            if let Some(msg_string) = msg_string_opt {
                return Ok(LuaValue::String(msg_string));
            }
        }
        Ok(LuaValue::Nil)
    }
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin + 'static,
{
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        let ws = Box::leak(Box::new(self));
        TableBuilder::new(lua)?
            .with_async_function("close", |_, code| ws.close(code))?
            .with_async_function("send", |_, (msg, bin)| ws.send_lua_string(msg, bin))?
            .with_async_function("next", |lua, _: ()| ws.next_lua_string(lua))?
            .with_metatable(
                TableBuilder::new(lua)?
                    .with_function(LuaMetaMethod::Index.name(), |_, key: String| {
                        if key == "closeCode" {
                            Ok(ws.get_lua_close_code())
                        } else {
                            Ok(LuaValue::Nil)
                        }
                    })?
                    .build_readonly()?,
            )?
            .build_readonly()
    }
}
