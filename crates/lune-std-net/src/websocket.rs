use std::sync::{
    atomic::{AtomicBool, AtomicU16, Ordering},
    Arc,
};

use bstr::{BString, ByteSlice};
use mlua::prelude::*;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex as AsyncMutex,
};

use hyper_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode as WsCloseCode, CloseFrame as WsCloseFrame},
        Message as WsMessage,
    },
    WebSocketStream,
};

#[derive(Debug)]
pub struct NetWebSocket<T> {
    close_code_exists: Arc<AtomicBool>,
    close_code_value: Arc<AtomicU16>,
    read_stream: Arc<AsyncMutex<SplitStream<WebSocketStream<T>>>>,
    write_stream: Arc<AsyncMutex<SplitSink<WebSocketStream<T>, WsMessage>>>,
}

impl<T> Clone for NetWebSocket<T> {
    fn clone(&self) -> Self {
        Self {
            close_code_exists: Arc::clone(&self.close_code_exists),
            close_code_value: Arc::clone(&self.close_code_value),
            read_stream: Arc::clone(&self.read_stream),
            write_stream: Arc::clone(&self.write_stream),
        }
    }
}

impl<T> NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin + 'static,
{
    pub fn new(value: WebSocketStream<T>) -> Self {
        let (write, read) = value.split();

        Self {
            close_code_exists: Arc::new(AtomicBool::new(false)),
            close_code_value: Arc::new(AtomicU16::new(0)),
            read_stream: Arc::new(AsyncMutex::new(read)),
            write_stream: Arc::new(AsyncMutex::new(write)),
        }
    }

    fn get_close_code(&self) -> Option<u16> {
        if self.close_code_exists.load(Ordering::Relaxed) {
            Some(self.close_code_value.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    fn set_close_code(&self, code: u16) {
        self.close_code_exists.store(true, Ordering::Relaxed);
        self.close_code_value.store(code, Ordering::Relaxed);
    }

    pub async fn send(&self, msg: WsMessage) -> LuaResult<()> {
        let mut ws = self.write_stream.lock().await;
        ws.send(msg).await.into_lua_err()
    }

    pub async fn next(&self) -> LuaResult<Option<WsMessage>> {
        let mut ws = self.read_stream.lock().await;
        ws.next().await.transpose().into_lua_err()
    }

    pub async fn close(&self, code: Option<u16>) -> LuaResult<()> {
        if self.close_code_exists.load(Ordering::Relaxed) {
            return Err(LuaError::runtime("Socket has already been closed"));
        }

        self.send(WsMessage::Close(Some(WsCloseFrame {
            code: match code {
                Some(code) if (1000..=4999).contains(&code) => WsCloseCode::from(code),
                Some(code) => {
                    return Err(LuaError::runtime(format!(
                        "Close code must be between 1000 and 4999, got {code}"
                    )))
                }
                None => WsCloseCode::Normal,
            },
            reason: "".into(),
        })))
        .await?;

        let mut ws = self.write_stream.lock().await;
        ws.close().await.into_lua_err()
    }
}

impl<T> LuaUserData for NetWebSocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin + 'static,
{
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("closeCode", |_, this| Ok(this.get_close_code()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("close", |_, this, code: Option<u16>| async move {
            this.close(code).await
        });

        methods.add_async_method(
            "send",
            |_, this, (string, as_binary): (BString, Option<bool>)| async move {
                this.send(if as_binary.unwrap_or_default() {
                    WsMessage::Binary(string.as_bytes().to_vec())
                } else {
                    let s = string.to_str().into_lua_err()?;
                    WsMessage::Text(s.to_string())
                })
                .await
            },
        );

        methods.add_async_method("next", |lua, this, (): ()| async move {
            let msg = this.next().await?;

            if let Some(WsMessage::Close(Some(frame))) = msg.as_ref() {
                this.set_close_code(frame.code.into());
            }

            Ok(match msg {
                Some(WsMessage::Binary(bin)) => LuaValue::String(lua.create_string(bin)?),
                Some(WsMessage::Text(txt)) => LuaValue::String(lua.create_string(txt)?),
                Some(WsMessage::Close(_)) | None => LuaValue::Nil,
                // Ignore ping/pong/frame messages, they are handled by tungstenite
                msg => unreachable!("Unhandled message: {:?}", msg),
            })
        });
    }
}
