use std::{collections::HashMap, str::FromStr};

use async_executor::Executor;
use http_body_util::Full;
use hyper::{
    body::Bytes,
    client::conn::http1::handshake,
    header::{HeaderName, HeaderValue, ACCEPT, CONTENT_LENGTH, HOST, USER_AGENT},
    Method, Request as HyperRequest,
};

use url::Url;

use crate::{
    client::http_stream::HttpStream,
    shared::{hyper::HyperIo, request::Request, response::Response},
};

/**
    Sends a simple request and returns the final response.

    This will follow any redirects returned by the server,
    modifying the request method and body as necessary.

    # WARNING

    This is an API meant only for private consumption by the main `lune`
    crate - unlike other functions in *this* crate, it is NOT guaranteed
    to follow semver or be otherwise stable outside of the private usage.
*/
#[doc(hidden)]
#[allow(clippy::implicit_hasher)]
pub async fn fetch(
    url: Url,
    method: Option<Method>,
    headers: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
) -> Result<Response, String> {
    let body = match body {
        Some(body) => Bytes::from(body),
        None => Bytes::new(),
    };

    let mut request = HyperRequest::new(body);
    *request.uri_mut() = url.to_string().parse().unwrap();
    if let Some(method) = method {
        *request.method_mut() = method;
    }
    if let Some(headers) = headers {
        for (key, val) in headers {
            let key = HeaderName::from_str(key.as_str()).map_err(|e| e.to_string())?;
            let val = HeaderValue::from_str(val.as_str()).map_err(|e| e.to_string())?;
            request.headers_mut().insert(key, val);
        }
    }

    // Some headers are required by most if not
    // all servers, make sure those are present...
    if !request.headers().contains_key(USER_AGENT.as_str()) {
        let ua = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let ua = HeaderValue::from_str(ua).unwrap();
        request.headers_mut().insert(USER_AGENT, ua);
    }
    if !request.headers().contains_key(CONTENT_LENGTH.as_str()) && request.method() != Method::GET {
        let len = request.body().len().to_string();
        let len = HeaderValue::from_str(&len).unwrap();
        request.headers_mut().insert(CONTENT_LENGTH, len);
    }
    if !request.headers().contains_key(ACCEPT.as_str()) {
        let accept = HeaderValue::from_static("*/*");
        request.headers_mut().insert(ACCEPT, accept);
    }

    // ... we can now safely continue and send the request
    let mut req = Request::from(request);
    req.decompress = true;

    let exec = Executor::new();
    let fut = fetch_inner(&exec, url, req);
    exec.run(fut).await
}

async fn fetch_inner(
    exec: &Executor<'_>,
    mut url: Url,
    mut request: Request,
) -> Result<Response, String> {
    loop {
        let stream = HttpStream::connect(url.clone())
            .await
            .map_err(|e| e.to_string())?;

        let (mut sender, conn) = handshake(HyperIo::from(stream))
            .await
            .map_err(|e| e.to_string())?;

        exec.spawn(conn).detach();

        let (mut parts, body) = request.clone_inner().into_parts();
        if let Some(host) = parts.uri.host() {
            let host = HeaderValue::from_str(host).unwrap();
            parts.headers.insert(HOST, host);
        }

        let data = HyperRequest::from_parts(parts, Full::new(body.into_bytes()));
        let incoming = sender.send_request(data).await.map_err(|e| e.to_string())?;

        if super::try_follow_redirect(&mut url, &mut request, &incoming)
            .map_err(ToString::to_string)?
        {
            continue;
        }

        break Response::from_incoming(incoming, request.decompress)
            .await
            .map_err(|e| e.to_string());
    }
}
