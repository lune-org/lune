use http_body_util::Full;
use hyper::{
    client::conn::http1::handshake,
    header::{HeaderValue, ACCEPT, CONTENT_LENGTH, HOST, USER_AGENT},
    Method, Request as HyperRequest,
};

use mlua::prelude::*;
use url::Url;

use crate::{
    client::http_stream::HttpStream,
    shared::{
        headers::create_user_agent_header,
        hyper::{HyperExecutor, HyperIo},
        request::Request,
        response::Response,
    },
};

/**
    Sends the request and returns the final response.

    This will follow any redirects returned by the server,
    modifying the request method and body as necessary.
*/
pub async fn send(mut request: Request, lua: Lua) -> LuaResult<Response> {
    let mut url = request
        .inner
        .uri()
        .to_string()
        .parse::<Url>()
        .into_lua_err()?;

    // Some headers are required by most if not
    // all servers, make sure those are present...
    if !request.headers().contains_key(USER_AGENT.as_str()) {
        let ua = create_user_agent_header(&lua)?;
        let ua = HeaderValue::from_str(&ua).unwrap();
        request.inner.headers_mut().insert(USER_AGENT, ua);
    }
    if !request.headers().contains_key(CONTENT_LENGTH.as_str()) && request.method() != Method::GET {
        let len = request.body().len().to_string();
        let len = HeaderValue::from_str(&len).unwrap();
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

        let (mut parts, body) = request.clone_inner().into_parts();
        if let Some(host) = parts.uri.host() {
            let host = HeaderValue::from_str(host).unwrap();
            parts.headers.insert(HOST, host);
        }

        let data = HyperRequest::from_parts(parts, Full::new(body.into_bytes()));
        let incoming = sender.send_request(data).await.into_lua_err()?;

        if super::try_follow_redirect(&mut url, &mut request, &incoming)
            .map_err(LuaError::external)?
        {
            continue;
        }

        break Response::from_incoming(incoming, request.decompress).await;
    }
}
