use std::{
    io::{Error, Result},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_net::TcpStream;
use async_tungstenite::{
    WebSocketStream as TungsteniteStream,
    tungstenite::{Error as TungsteniteError, Message, Result as TungsteniteResult},
};
use futures::Sink;
use futures_lite::prelude::*;
use futures_rustls::{TlsConnector, TlsStream};
use rustls_pki_types::ServerName;
use url::Url;

use crate::client::rustls::CLIENT_CONFIG;

/**
    Type alias for differentiating between a [`MaybeTlsStream`]
    from a [`WsStream`] when using HTTP a bit clearer.
*/
pub type HttpStream = MaybeTlsStream;

/**
    A TCP stream that may or may not be encrypted using TLS.

    Implements both `AsyncRead` and `AsyncWrite` such that
    any consumers of this stream do not need to care about
    the inner TLS-or-not stream and any associated details.
*/
#[derive(Debug)]
pub enum MaybeTlsStream {
    Plain(Box<TcpStream>),
    Tls(Box<TlsStream<TcpStream>>),
}

impl MaybeTlsStream {
    /**
        Connects to a host and port, additionally using TLS if specified.

        Using this constructor is likely unergonomic - prefer using
        [`MaybeTlsStream::connect_url`] instead, if possible.

        The given `host` must be a valid DNS name, when using TLS.
    */
    pub async fn connect(host: &str, port: u16, tls: bool) -> Result<Self> {
        let stream = TcpStream::connect((host, port)).await?;

        let stream = if tls {
            let servname = ServerName::try_from(host).map_err(Error::other)?.to_owned();
            let connector = TlsConnector::from(Arc::clone(&CLIENT_CONFIG));
            let stream = connector.connect(servname, stream).await?;
            Self::Tls(Box::new(TlsStream::Client(stream)))
        } else {
            Self::Plain(Box::new(stream))
        };

        Ok(stream)
    }

    /**
       Connects to the given URL.

       Automatically determines whether or not to use TLS based on the URL scheme.
    */
    pub async fn connect_url(url: Url) -> Result<Self> {
        let Some(host) = url.host() else {
            return Err(Error::other("unknown or missing host"));
        };
        let Some(port) = url.port_or_known_default() else {
            return Err(Error::other("unknown or missing port"));
        };

        let use_tls = match url.scheme() {
            "http" | "ws" => false,
            "https" | "wss" => true,
            s => return Err(Error::other(format!("unsupported scheme: {s}"))),
        };

        let host = host.to_string();
        Self::connect(&host, port, use_tls).await
    }

    /**
        Returns the local address of the stream.
    */
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.as_ref().local_addr()
    }

    /**
        Returns the remote address of the stream.
    */
    pub fn remote_addr(&self) -> Result<SocketAddr> {
        self.as_ref().peer_addr()
    }

    /**
        Sets the TTL (Time To Live) for packets in the stream.

        See [`TcpStream::set_ttl`] for additional information.
    */
    pub fn set_ttl(&self, ttl: u32) -> Result<()> {
        self.as_ref().set_ttl(ttl)
    }
}

impl AsRef<TcpStream> for MaybeTlsStream {
    fn as_ref(&self) -> &TcpStream {
        match self {
            MaybeTlsStream::Plain(stream) => stream,
            MaybeTlsStream::Tls(stream) => stream.get_ref().0,
        }
    }
}

impl From<TcpStream> for MaybeTlsStream {
    fn from(stream: TcpStream) -> Self {
        MaybeTlsStream::Plain(Box::new(stream))
    }
}

impl From<TlsStream<TcpStream>> for MaybeTlsStream {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        MaybeTlsStream::Tls(Box::new(stream))
    }
}

impl AsyncRead for MaybeTlsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            MaybeTlsStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for MaybeTlsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            MaybeTlsStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match &mut *self {
            MaybeTlsStream::Plain(stream) => Pin::new(stream).poll_close(cx),
            MaybeTlsStream::Tls(stream) => Pin::new(stream).poll_close(cx),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match &mut *self {
            MaybeTlsStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
            MaybeTlsStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }
}

/**
    A WebSocket stream.

    Implements both `Sink` and `Stream` traits.
*/
#[derive(Debug)]
pub struct WsStream {
    inner: TungsteniteStream<MaybeTlsStream>,
}

impl WsStream {
    /**
       Connects to the given URL.

       Automatically determines whether or not to use TLS based on the URL scheme.
    */
    pub async fn connect_url(url: Url) -> Result<Self> {
        let url_str = url.to_string();

        let stream = MaybeTlsStream::connect_url(url).await?;
        let (inner, _) = async_tungstenite::client_async(url_str, stream)
            .await
            .map_err(Error::other)?;

        Ok(Self { inner })
    }
}

impl Sink<Message> for WsStream {
    type Error = TungsteniteError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<TungsteniteResult<()>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> TungsteniteResult<()> {
        Pin::new(&mut self.inner).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<TungsteniteResult<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<TungsteniteResult<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl Stream for WsStream {
    type Item = TungsteniteResult<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
