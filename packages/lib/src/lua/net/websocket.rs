use std::{cell::Cell, sync::Arc};

use hyper::upgrade::Upgraded;
use mlua::prelude::*;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    sync::Mutex as AsyncMutex,
};

use hyper_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode as WsCloseCode, CloseFrame as WsCloseFrame},
        Message as WsMessage,
    },
    WebSocketStream,
};
use tokio_tungstenite::MaybeTlsStream;

use crate::lua::table::TableBuilder;

const WEB_SOCKET_IMPL_LUA: &str = r#"
return freeze(setmetatable({
	close = function(...)
		return close(websocket, ...)
	end,
	send = function(...)
		return send(websocket, ...)
	end,
	next = function(...)
		return next(websocket, ...)
	end,
}, {
	__index = function(self, key)
		if key == "closeCode" then
			return close_code(websocket)
		end
	end,
}))
"#;

#[derive(Debug)]
pub struct NetWebSocket<T> {
    close_code: Arc<Cell<Option<u16>>>,
    stream_in: Arc<AsyncMutex<SplitStream<WebSocketStream<T>>>>,
    stream_out: Arc<AsyncMutex<SplitSink<WebSocketStream<T>, WsMessage>>>,
}

impl<T> Clone for NetWebSocket<T> {
    fn clone(&self) -> Self {
        Self {
            close_code: Arc::clone(&self.close_code),
            stream_in: Arc::clone(&self.stream_in),
            stream_out: Arc::clone(&self.stream_out),
        }
    }
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(value: WebSocketStream<T>) -> Self {
        let (write, read) = value.split();

        Self {
            close_code: Arc::new(Cell::new(None)),
            stream_in: Arc::new(AsyncMutex::new(read)),
            stream_out: Arc::new(AsyncMutex::new(write)),
        }
    }

    fn into_lua_table_with_env<'lua>(
        lua: &'lua Lua,
        env: LuaTable<'lua>,
    ) -> LuaResult<LuaTable<'lua>> {
        lua.load(WEB_SOCKET_IMPL_LUA)
            .set_name("websocket")
            .set_environment(env)
            .eval()
    }
}

type NetWebSocketStreamClient = MaybeTlsStream<TcpStream>;
impl NetWebSocket<NetWebSocketStreamClient> {
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        let socket_env = TableBuilder::new(lua)?
            .with_value("websocket", self)?
            .with_function("close_code", close_code::<NetWebSocketStreamClient>)?
            .with_async_function("close", close::<NetWebSocketStreamClient>)?
            .with_async_function("send", send::<NetWebSocketStreamClient>)?
            .with_async_function("next", next::<NetWebSocketStreamClient>)?
            .with_value(
                "setmetatable",
                lua.named_registry_value::<LuaFunction>("tab.setmeta")?,
            )?
            .with_value(
                "freeze",
                lua.named_registry_value::<LuaFunction>("tab.freeze")?,
            )?
            .build_readonly()?;
        Self::into_lua_table_with_env(lua, socket_env)
    }
}

type NetWebSocketStreamServer = Upgraded;
impl NetWebSocket<NetWebSocketStreamServer> {
    pub fn into_lua_table(self, lua: &'static Lua) -> LuaResult<LuaTable> {
        let socket_env = TableBuilder::new(lua)?
            .with_value("websocket", self)?
            .with_function("close_code", close_code::<NetWebSocketStreamServer>)?
            .with_async_function("close", close::<NetWebSocketStreamServer>)?
            .with_async_function("send", send::<NetWebSocketStreamServer>)?
            .with_async_function("next", next::<NetWebSocketStreamServer>)?
            .with_value(
                "setmetatable",
                lua.named_registry_value::<LuaFunction>("tab.setmeta")?,
            )?
            .with_value(
                "freeze",
                lua.named_registry_value::<LuaFunction>("tab.freeze")?,
            )?
            .build_readonly()?;
        Self::into_lua_table_with_env(lua, socket_env)
    }
}

impl<T> LuaUserData for NetWebSocket<T> {}

fn close_code<'lua, T>(
    _lua: &'lua Lua,
    socket: LuaUserDataRef<'lua, NetWebSocket<T>>,
) -> LuaResult<LuaValue<'lua>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    Ok(match socket.close_code.get() {
        Some(code) => LuaValue::Number(code as f64),
        None => LuaValue::Nil,
    })
}

async fn close<'lua, T>(
    _lua: &'lua Lua,
    (socket, code): (LuaUserDataRef<'lua, NetWebSocket<T>>, Option<u16>),
) -> LuaResult<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut ws = socket.stream_out.lock().await;

    let _ = ws
        .send(WsMessage::Close(Some(WsCloseFrame {
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
        })))
        .await;

    let res = ws.close();
    res.await.map_err(LuaError::external)
}

async fn send<'lua, T>(
    _lua: &'lua Lua,
    (socket, string, as_binary): (
        LuaUserDataRef<'lua, NetWebSocket<T>>,
        LuaString<'lua>,
        Option<bool>,
    ),
) -> LuaResult<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let msg = if matches!(as_binary, Some(true)) {
        WsMessage::Binary(string.as_bytes().to_vec())
    } else {
        let s = string.to_str().map_err(LuaError::external)?;
        WsMessage::Text(s.to_string())
    };
    let mut ws = socket.stream_out.lock().await;
    ws.send(msg).await.map_err(LuaError::external)
}

async fn next<'lua, T>(
    lua: &'lua Lua,
    socket: LuaUserDataRef<'lua, NetWebSocket<T>>,
) -> LuaResult<LuaValue<'lua>>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut ws = socket.stream_in.lock().await;
    let item = ws.next().await.transpose().map_err(LuaError::external);
    let msg = match item {
        Ok(Some(WsMessage::Close(msg))) => {
            if let Some(msg) = &msg {
                socket.close_code.replace(Some(msg.code.into()));
            }
            Ok(Some(WsMessage::Close(msg)))
        }
        val => val,
    }?;
    while let Some(msg) = &msg {
        let msg_string_opt = match msg {
            WsMessage::Binary(bin) => Some(lua.create_string(bin)?),
            WsMessage::Text(txt) => Some(lua.create_string(txt)?),
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
