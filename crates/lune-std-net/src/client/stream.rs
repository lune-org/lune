use std::{
    io,
    pin::Pin,
    sync::{Arc, LazyLock},
    task::{Context, Poll},
};

use async_net::TcpStream;
use futures_lite::prelude::*;
use futures_rustls::{TlsConnector, TlsStream};
use hyper::Uri;
use rustls::ClientConfig;
use rustls_pki_types::ServerName;

static CLIENT_CONFIG: LazyLock<Arc<ClientConfig>> = LazyLock::new(|| {
    rustls::ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        })
        .with_no_client_auth()
        .into()
});

pub enum HttpRequestStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl HttpRequestStream {
    pub async fn connect(url: Uri) -> Result<Self, io::Error> {
        let Some(host) = url.host() else {
            return Err(make_err("unknown or missing host"));
        };
        let Some(scheme) = url.scheme_str() else {
            return Err(make_err("unknown scheme"));
        };

        let (use_tls, port) = match scheme {
            "http" => (false, 80),
            "https" => (true, 443),
            s => return Err(make_err(format!("unsupported scheme: {s}"))),
        };

        let stream = {
            let port = url.port_u16().unwrap_or(port);
            TcpStream::connect((host, port)).await?
        };

        let stream = if use_tls {
            let servname = ServerName::try_from(host).map_err(make_err)?.to_owned();
            let connector = TlsConnector::from(Arc::clone(&CLIENT_CONFIG));
            let stream = connector.connect(servname, stream).await?;
            Self::Tls(TlsStream::Client(stream))
        } else {
            Self::Plain(stream)
        };

        Ok(stream)
    }
}

impl AsyncRead for HttpRequestStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            HttpRequestStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            HttpRequestStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for HttpRequestStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            HttpRequestStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            HttpRequestStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            HttpRequestStream::Plain(stream) => Pin::new(stream).poll_close(cx),
            HttpRequestStream::Tls(stream) => Pin::new(stream).poll_close(cx),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            HttpRequestStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
            HttpRequestStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }
}

fn make_err(e: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}
