use hyper::{Method, Response as HyperResponse, Uri, body::Incoming, header::LOCATION};

use mlua::prelude::*;
use url::Url;

use crate::{
    body::ReadableBody,
    client::{
        stream::{MaybeTlsStream, WsStream},
        tcp::TcpConfig,
    },
    shared::{request::Request, tcp::Tcp, websocket::Websocket},
};

pub mod rustls;
pub mod stream;
pub mod tcp;

mod fetch;
mod send;

pub use self::fetch::fetch;
pub use self::send::send;

const MAX_REDIRECTS: usize = 10;

/**
    Connects to a websocket at the given URL.
*/
pub async fn connect_ws(url: Url) -> LuaResult<Websocket<WsStream>> {
    let stream = WsStream::connect_url(url).await?;
    Ok(Websocket::from(stream))
}

/**
    Connects using plain TCP using the given host, port, and config.
*/
pub async fn connect_tcp(host: String, port: u16, config: TcpConfig) -> LuaResult<Tcp> {
    let tls = config.tls.unwrap_or_default();

    let stream = MaybeTlsStream::connect(&host, port, tls)
        .await
        .into_lua_err()?;

    if let Some(ttl) = config.ttl {
        stream.set_ttl(ttl).into_lua_err()?;
    }

    Ok(Tcp::from(stream))
}

fn try_follow_redirect(
    url: &mut Url,
    request: &mut Request,
    response: &HyperResponse<Incoming>,
) -> Result<bool, &'static str> {
    if let Some((new_method, new_uri)) = check_redirect(request.inner.method().clone(), response) {
        if request.redirects.is_some_and(|r| r >= MAX_REDIRECTS) {
            return Err("Too many redirects");
        }

        if new_uri.host().is_some() {
            let new_url = new_uri
                .to_string()
                .parse()
                .map_err(|_| "Invalid redirect URL")?;
            *url = new_url;
        } else {
            url.set_path(new_uri.path());
        }

        if new_method == Method::GET {
            *request.inner.body_mut() = ReadableBody::empty();
        }

        *request.inner.method_mut() = new_method;
        *request.inner.uri_mut() = new_uri;

        *request.redirects.get_or_insert_default() += 1;

        Ok(true)
    } else {
        Ok(false)
    }
}

fn check_redirect(method: Method, response: &HyperResponse<Incoming>) -> Option<(Method, Uri)> {
    if !response.status().is_redirection() {
        return None;
    }

    let location = response.headers().get(LOCATION)?;
    let location = location.to_str().ok()?;
    let location = location.parse().ok()?;

    let method = match response.status().as_u16() {
        301..=303 => Method::GET,
        _ => method,
    };

    Some((method, location))
}
