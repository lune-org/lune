use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};

use async_lock::Mutex as AsyncMutex;
use async_tungstenite::tungstenite::{
    Message as TungsteniteMessage, Result as TungsteniteResult, Utf8Bytes,
    protocol::{CloseFrame, frame::coding::CloseCode},
};
use bstr::{BString, ByteSlice};
use futures::{
    Sink, SinkExt, Stream, StreamExt,
    stream::{SplitSink, SplitStream},
};
use hyper::body::Bytes;

use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct Websocket<T> {
    close_code_exists: Arc<AtomicBool>,
    close_code_value: Arc<AtomicU16>,
    read_stream: Arc<AsyncMutex<SplitStream<T>>>,
    write_stream: Arc<AsyncMutex<SplitSink<T, TungsteniteMessage>>>,
}

impl<T> Websocket<T>
where
    T: Stream<Item = TungsteniteResult<TungsteniteMessage>> + Sink<TungsteniteMessage> + 'static,
    <T as Sink<TungsteniteMessage>>::Error: Into<Box<dyn Error + Send + Sync + 'static>>,
{
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

    pub async fn send(&self, msg: TungsteniteMessage) -> LuaResult<()> {
        let mut ws = self.write_stream.lock().await;
        ws.send(msg).await.into_lua_err()
    }

    pub async fn next(&self) -> LuaResult<Option<TungsteniteMessage>> {
        let mut ws = self.read_stream.lock().await;
        ws.next().await.transpose().into_lua_err()
    }

    pub async fn close(&self, code: Option<u16>) -> LuaResult<()> {
        if self.close_code_exists.load(Ordering::Relaxed) {
            return Err(LuaError::runtime("Socket has already been closed"));
        }

        self.send(TungsteniteMessage::Close(Some(CloseFrame {
            code: match code {
                Some(code) if (1000..=4999).contains(&code) => CloseCode::from(code),
                Some(code) => {
                    return Err(LuaError::runtime(format!(
                        "Close code must be between 1000 and 4999, got {code}"
                    )));
                }
                None => CloseCode::Normal,
            },
            reason: "".into(),
        })))
        .await?;

        let mut ws = self.write_stream.lock().await;
        ws.close().await.into_lua_err()
    }
}

impl<T> From<T> for Websocket<T>
where
    T: Stream<Item = TungsteniteResult<TungsteniteMessage>> + Sink<TungsteniteMessage> + 'static,
    <T as Sink<TungsteniteMessage>>::Error: Into<Box<dyn Error + Send + Sync + 'static>>,
{
    fn from(value: T) -> Self {
        let (write, read) = value.split();

        Self {
            close_code_exists: Arc::new(AtomicBool::new(false)),
            close_code_value: Arc::new(AtomicU16::new(0)),
            read_stream: Arc::new(AsyncMutex::new(read)),
            write_stream: Arc::new(AsyncMutex::new(write)),
        }
    }
}

impl<T> LuaUserData for Websocket<T>
where
    T: Stream<Item = TungsteniteResult<TungsteniteMessage>> + Sink<TungsteniteMessage> + 'static,
    <T as Sink<TungsteniteMessage>>::Error: Into<Box<dyn Error + Send + Sync + 'static>>,
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
                    TungsteniteMessage::Binary(Bytes::from(string.to_vec()))
                } else {
                    let s = string.to_str().into_lua_err()?;
                    TungsteniteMessage::Text(Utf8Bytes::from(s))
                })
                .await
            },
        );

        methods.add_async_method("next", |lua, this, (): ()| async move {
            let msg = this.next().await?;

            if let Some(TungsteniteMessage::Close(Some(frame))) = msg.as_ref() {
                this.set_close_code(frame.code.into());
            }

            Ok(match msg {
                Some(TungsteniteMessage::Binary(bin)) => LuaValue::String(lua.create_string(bin)?),
                Some(TungsteniteMessage::Text(txt)) => LuaValue::String(lua.create_string(txt)?),
                Some(TungsteniteMessage::Close(_)) | None => LuaValue::Nil,
                // Ignore ping/pong/frame messages, they are handled by tungstenite
                msg => unreachable!("Unhandled message: {:?}", msg),
            })
        });
    }
}
