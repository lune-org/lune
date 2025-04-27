use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_net::TcpStream;
use async_tungstenite::{
    tungstenite::{Error as TungsteniteError, Message, Result as TungsteniteResult},
    WebSocketStream as TungsteniteStream,
};
use futures::Sink;
use futures_lite::prelude::*;
use futures_rustls::{TlsConnector, TlsStream};
use rustls_pki_types::ServerName;
use url::Url;

use crate::client::rustls::CLIENT_CONFIG;

#[derive(Debug)]
pub enum WsStream {
    Plain(TungsteniteStream<TcpStream>),
    Tls(TungsteniteStream<TlsStream<TcpStream>>),
}

impl WsStream {
    pub async fn connect(url: Url) -> Result<Self, io::Error> {
        let Some(host) = url.host() else {
            return Err(make_err("unknown or missing host"));
        };
        let Some(port) = url.port_or_known_default() else {
            return Err(make_err("unknown or missing port"));
        };

        let use_tls = match url.scheme() {
            "ws" => false,
            "wss" => true,
            s => return Err(make_err(format!("unsupported scheme: {s}"))),
        };

        let host = host.to_string();
        let stream = TcpStream::connect((host.clone(), port)).await?;

        let stream = if use_tls {
            let servname = ServerName::try_from(host).map_err(make_err)?.to_owned();
            let connector = TlsConnector::from(Arc::clone(&CLIENT_CONFIG));

            let stream = connector.connect(servname, stream).await?;
            let stream = TlsStream::Client(stream);

            let stream = async_tungstenite::client_async(url.to_string(), stream)
                .await
                .map_err(make_err)?
                .0;
            Self::Tls(stream)
        } else {
            let stream = async_tungstenite::client_async(url.to_string(), stream)
                .await
                .map_err(make_err)?
                .0;
            Self::Plain(stream)
        };

        Ok(stream)
    }
}

impl Sink<Message> for WsStream {
    type Error = TungsteniteError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            WsStream::Plain(s) => Pin::new(s).poll_ready(cx),
            WsStream::Tls(s) => Pin::new(s).poll_ready(cx),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match &mut *self {
            WsStream::Plain(s) => Pin::new(s).start_send(item),
            WsStream::Tls(s) => Pin::new(s).start_send(item),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            WsStream::Plain(s) => Pin::new(s).poll_flush(cx),
            WsStream::Tls(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            WsStream::Plain(s) => Pin::new(s).poll_close(cx),
            WsStream::Tls(s) => Pin::new(s).poll_close(cx),
        }
    }
}

impl Stream for WsStream {
    type Item = TungsteniteResult<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut *self {
            WsStream::Plain(s) => Pin::new(s).poll_next(cx),
            WsStream::Tls(s) => Pin::new(s).poll_next(cx),
        }
    }
}

fn make_err(e: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}
