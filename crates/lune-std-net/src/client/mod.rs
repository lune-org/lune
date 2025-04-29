use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::handshake,
    header::{HeaderValue, ACCEPT, CONTENT_LENGTH, HOST, LOCATION, USER_AGENT},
    Method, Request as HyperRequest, Response as HyperResponse, Uri,
};

use mlua::prelude::*;
use url::Url;

use crate::{
    client::{http_stream::HttpStream, ws_stream::WsStream},
    shared::{
        headers::create_user_agent_header,
        hyper::{HyperExecutor, HyperIo},
        request::Request,
        response::Response,
        websocket::Websocket,
    },
};

pub mod http_stream;
pub mod rustls;
pub mod ws_stream;

const MAX_REDIRECTS: usize = 10;

/**
    Connects to a websocket at the given URL.
*/
pub async fn connect_websocket(url: Url) -> LuaResult<Websocket<WsStream>> {
    let stream = WsStream::connect(url).await?;
    Ok(Websocket::from(stream))
}

/**
    Sends the request and returns the final response.

    This will follow any redirects returned by the server,
    modifying the request method and body as necessary.
*/
pub async fn send_request(mut request: Request, lua: Lua) -> LuaResult<Response> {
    let url = request
        .inner
        .uri()
        .to_string()
        .parse::<Url>()
        .expect("uri is valid");

    // Some headers are required by most if not
    // all servers, make sure those are present...
    if !request.headers().contains_key(HOST.as_str()) {
        if let Some(host) = url.host_str() {
            let host = HeaderValue::from_str(host).into_lua_err()?;
            request.inner.headers_mut().insert(HOST, host);
        }
    }
    if !request.headers().contains_key(USER_AGENT.as_str()) {
        let ua = create_user_agent_header(&lua)?;
        let ua = HeaderValue::from_str(&ua).into_lua_err()?;
        request.inner.headers_mut().insert(USER_AGENT, ua);
    }
    if !request.headers().contains_key(CONTENT_LENGTH.as_str()) && request.method() != Method::GET {
        let len = request.inner.body().len().to_string();
        let len = HeaderValue::from_str(&len).into_lua_err()?;
        request.inner.headers_mut().insert(CONTENT_LENGTH, len);
    }
    if !request.headers().contains_key(ACCEPT.as_str()) {
        let accept = HeaderValue::from_static("*/*");
        request.inner.headers_mut().insert(ACCEPT, accept);
    }

    // ... we can now safely continue and send the request
    loop {
        let stream = HttpStream::connect(url.clone()).await?;

        let (mut sender, conn) = handshake(HyperIo::from(stream)).await.into_lua_err()?;

        HyperExecutor::execute(lua.clone(), conn);

        let incoming = sender
            .send_request(request.as_full())
            .await
            .into_lua_err()?;

        if let Some((new_method, new_uri)) = check_redirect(&request.inner, &incoming) {
            if request.redirects.is_some_and(|r| r >= MAX_REDIRECTS) {
                return Err(LuaError::external("Too many redirects"));
            }

            if new_method == Method::GET {
                *request.inner.body_mut() = Bytes::new();
            }

            *request.inner.method_mut() = new_method;
            *request.inner.uri_mut() = new_uri;

            *request.redirects.get_or_insert_default() += 1;

            continue;
        }

        break Response::from_incoming(incoming, request.decompress).await;
    }
}

fn check_redirect(
    request: &HyperRequest<Bytes>,
    response: &HyperResponse<Incoming>,
) -> Option<(Method, Uri)> {
    if !response.status().is_redirection() {
        return None;
    }

    let location = response.headers().get(LOCATION)?;
    let location = location.to_str().ok()?;
    let location = location.parse().ok()?;

    let method = match response.status().as_u16() {
        301..=303 => Method::GET,
        _ => request.method().clone(),
    };

    Some((method, location))
}
