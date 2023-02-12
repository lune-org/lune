use std::sync::Arc;

use mlua::prelude::*;

use hyper_tungstenite::{tungstenite::Message as WsMessage, WebSocketStream};

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::MaybeTlsStream;

#[derive(Debug, Clone)]
pub struct NetWebSocketClient(Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>);

impl NetWebSocketClient {
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

    pub async fn into_proper(self, lua: &'static Lua) -> LuaResult<LuaAnyUserData> {
        // HACK: This creates a new userdata that consumes and proxies this one,
        // since there's no great way to implement this in pure async Rust
        // and as a plain table without tons of strange lifetime issues
        let chunk = r#"
        local ws = ...
        local proxy = newproxy(true)
        local meta = getmetatable(proxy)
        meta.__index = {
            close = function()    return ws:close()   end,
            send  = function(...) return ws:send(...) end,
            next  = function()    return ws:next()    end,
        }
        meta.__iter = function()
            return function()
                return ws:next()
            end
        end
        return proxy
        "#;
        lua.load(chunk).call_async(self).await
    }
}

impl LuaUserData for NetWebSocketClient {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("close", |_, this, _: ()| async move { this.close().await });
        methods.add_async_method("send", |_, this, msg: String| async move {
            this.send(WsMessage::Text(msg)).await
        });
        methods.add_async_method("next", |lua, this, _: ()| async move {
            match this.next().await? {
                Some(msg) => Ok(match msg {
                    WsMessage::Binary(bin) => LuaValue::String(lua.create_string(&bin)?),
                    WsMessage::Text(txt) => LuaValue::String(lua.create_string(&txt)?),
                    _ => LuaValue::Nil,
                }),
                None => Ok(LuaValue::Nil),
            }
        });
    }
}

impl From<WebSocketStream<MaybeTlsStream<TcpStream>>> for NetWebSocketClient {
    fn from(value: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
