use hyper::{body::Incoming, header::LOCATION, Method, Response as HyperResponse, Uri};

use mlua::prelude::*;
use url::Url;

use crate::{
    body::ReadableBody,
    client::stream::WsStream,
    shared::{request::Request, websocket::Websocket},
};

pub mod rustls;
pub mod stream;

mod fetch;
mod send;

pub use self::fetch::fetch;
pub use self::send::send;

const MAX_REDIRECTS: usize = 10;

/**
    Connects to a websocket at the given URL.
*/
pub async fn connect_websocket(url: Url) -> LuaResult<Websocket<WsStream>> {
    let stream = WsStream::connect_url(url).await?;
    Ok(Websocket::from(stream))
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
