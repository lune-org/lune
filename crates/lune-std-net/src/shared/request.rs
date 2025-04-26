use http_body_util::Full;

use hyper::{
    body::Bytes,
    client::conn::http1::handshake,
    header::{HeaderName, HeaderValue, USER_AGENT},
    HeaderMap, Request as HyperRequest,
};

use mlua::prelude::*;
use url::Url;

use crate::{
    client::{config::RequestConfig, stream::HttpRequestStream},
    shared::{
        headers::create_user_agent_header,
        hyper::{HyperExecutor, HyperIo},
        response::Response,
    },
};

#[derive(Debug, Clone)]
pub struct Request {
    inner: HyperRequest<Full<Bytes>>,
}

impl Request {
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
        Ok(Self { inner })
    }

    pub async fn send(self, lua: Lua) -> LuaResult<Response> {
        let stream = HttpRequestStream::connect(self.inner.uri()).await?;

        let (mut sender, conn) = handshake(HyperIo::from(stream))
            .await
            .map_err(LuaError::external)?;

        HyperExecutor::execute(lua, conn);

        let incoming = sender
            .send_request(self.inner)
            .await
            .map_err(LuaError::external)?;

        Response::from_incoming(incoming).await
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
