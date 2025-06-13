use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_net::TcpStream;
use futures_lite::prelude::*;
use futures_rustls::{TlsConnector, TlsStream};
use rustls_pki_types::ServerName;
use url::Url;

use crate::client::rustls::CLIENT_CONFIG;

#[derive(Debug)]
pub enum HttpStream {
    Plain(Box<TcpStream>),
    Tls(Box<TlsStream<TcpStream>>),
}

impl HttpStream {
    pub async fn connect(url: Url) -> Result<Self, io::Error> {
        let Some(host) = url.host() else {
            return Err(make_err("unknown or missing host"));
        };
        let Some(port) = url.port_or_known_default() else {
            return Err(make_err("unknown or missing port"));
        };

        let use_tls = match url.scheme() {
            "http" => false,
            "https" => true,
            s => return Err(make_err(format!("unsupported scheme: {s}"))),
        };

        let host = host.to_string();
        let stream = TcpStream::connect((host.clone(), port)).await?;

        let stream = if use_tls {
            let servname = ServerName::try_from(host).map_err(make_err)?.to_owned();
            let connector = TlsConnector::from(Arc::clone(&CLIENT_CONFIG));
            let stream = connector.connect(servname, stream).await?;
            Self::Tls(Box::new(TlsStream::Client(stream)))
        } else {
            Self::Plain(Box::new(stream))
        };

        Ok(stream)
    }
}

impl AsyncRead for HttpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            HttpStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            HttpStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for HttpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            HttpStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            HttpStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            HttpStream::Plain(stream) => Pin::new(stream).poll_close(cx),
            HttpStream::Tls(stream) => Pin::new(stream).poll_close(cx),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            HttpStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
            HttpStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }
}

fn make_err(e: impl ToString) -> io::Error {
    io::Error::other(e.to_string())
}
