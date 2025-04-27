use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::handshake,
    header::LOCATION,
    Method, Request as HyperRequest, Response as HyperResponse, Uri,
};

use mlua::prelude::*;

use crate::{
    client::stream::HttpRequestStream,
    shared::{
        hyper::{HyperExecutor, HyperIo},
        request::Request,
        response::Response,
    },
};

pub mod config;
pub mod stream;

const MAX_REDIRECTS: usize = 10;

/**
    Sends the request and returns the final response.

    This will follow any redirects returned by the server,
    modifying the request method and body as necessary.
*/
pub async fn send_request(mut request: Request, lua: Lua) -> LuaResult<Response> {
    loop {
        let stream = HttpRequestStream::connect(request.inner.uri()).await?;

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
