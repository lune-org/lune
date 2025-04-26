use http_body_util::Full;
use url::Url;

use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::handshake,
    header::{HeaderName, HeaderValue, LOCATION, USER_AGENT},
    HeaderMap, Method, Request as HyperRequest, Response as HyperResponse, Uri,
};

use mlua::prelude::*;

use crate::{
    client::{config::RequestConfig, stream::HttpRequestStream},
    shared::{
        headers::create_user_agent_header,
        hyper::{HyperExecutor, HyperIo},
        response::Response,
    },
};

const MAX_REDIRECTS: usize = 10;

#[derive(Debug, Clone)]
pub struct Request {
    inner: HyperRequest<Full<Bytes>>,
    redirects: usize,
    decompress: bool,
}

impl Request {
    /**
        Creates a new request that is ready to be sent from a request configuration.
    */
    pub fn from_config(config: RequestConfig, lua: Lua) -> LuaResult<Self> {
        // 1. Parse the URL and make sure it is valid
        let mut url = Url::parse(&config.url).into_lua_err()?;

        // 2. Append any query pairs passed as a table
        {
            let mut query = url.query_pairs_mut();
            for (key, values) in config.query {
                for value in values {
                    query.append_pair(&key, &value);
                }
            }
        }

        // 3. Create the inner request builder
        let mut builder = HyperRequest::builder()
            .method(config.method)
            .uri(url.as_str());

        // 4. Append any headers passed as a table - builder
        //    headers may be None if builder is already invalid
        if let Some(headers) = builder.headers_mut() {
            for (key, values) in config.headers {
                let key = HeaderName::from_bytes(key.as_bytes()).into_lua_err()?;
                for value in values {
                    let value = HeaderValue::from_str(&value).into_lua_err()?;
                    headers.insert(key.clone(), value);
                }
            }
        }

        // 5. Convert request body bytes to the proper Body
        //    type that Hyper expects, if we got any bytes
        let body = config
            .body
            .map(Bytes::from)
            .map(Full::new)
            .unwrap_or_default();

        // 6. Finally, attach the body, verifying that the request
        //    is valid, and attach a user agent if not already set
        let mut inner = builder.body(body).into_lua_err()?;

        add_default_headers(&lua, inner.headers_mut())?;

        Ok(Self {
            inner,
            redirects: 0,
            decompress: config.options.decompress,
        })
    }

    /**
        Sends the request and returns the final response.

        This will follow any redirects returned by the server,
        modifying the request method and body as necessary.
    */
    pub async fn send(mut self, lua: Lua) -> LuaResult<Response> {
        loop {
            let stream = HttpRequestStream::connect(self.inner.uri()).await?;

            let (mut sender, conn) = handshake(HyperIo::from(stream))
                .await
                .map_err(LuaError::external)?;

            HyperExecutor::execute(lua.clone(), conn);

            let incoming = sender
                .send_request(self.inner.clone())
                .await
                .map_err(LuaError::external)?;

            if let Some((replacement_method, replacement_uri)) =
                check_redirect(&self.inner, &incoming)
            {
                if self.redirects >= MAX_REDIRECTS {
                    return Err(LuaError::external("Too many redirects"));
                }

                if replacement_method == Method::GET {
                    *self.inner.body_mut() = Full::default();
                }

                *self.inner.method_mut() = replacement_method;
                *self.inner.uri_mut() = replacement_uri;

                self.redirects += 1;

                continue;
            }

            break Response::from_incoming(incoming, self.decompress).await;
        }
    }
}

fn add_default_headers(lua: &Lua, headers: &mut HeaderMap) -> LuaResult<()> {
    if !headers.contains_key(USER_AGENT) {
        let ua = create_user_agent_header(lua)?;
        let ua = HeaderValue::from_str(&ua).into_lua_err()?;
        headers.insert(USER_AGENT, ua);
    }

    Ok(())
}

fn check_redirect(
    request: &HyperRequest<Full<Bytes>>,
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
