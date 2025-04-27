use std::collections::HashMap;

use futures_lite::prelude::*;
use http_body_util::{BodyStream, Full};
use url::Url;

use hyper::{
    body::{Body as _, Bytes, Incoming},
    header::{HeaderName, HeaderValue, USER_AGENT},
    HeaderMap, Method, Request as HyperRequest,
};

use mlua::prelude::*;

use crate::{
    client::config::RequestConfig,
    shared::headers::{create_user_agent_header, hash_map_to_table, header_map_to_table},
};

#[derive(Debug, Clone)]
pub struct Request {
    // NOTE: We use Bytes instead of Full<Bytes> to avoid
    // needing async when getting a reference to the body
    pub(crate) inner: HyperRequest<Bytes>,
    pub(crate) redirects: usize,
    pub(crate) decompress: bool,
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
        let body = config.body.map(Bytes::from).unwrap_or_default();

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
        Creates a new request from a raw incoming request.
    */
    pub async fn from_incoming(
        incoming: HyperRequest<Incoming>,
        decompress: bool,
    ) -> LuaResult<Self> {
        let (parts, body) = incoming.into_parts();

        let size = body.size_hint().lower() as usize;
        let buffer = Vec::<u8>::with_capacity(size);
        let body = BodyStream::new(body)
            .try_fold(buffer, |mut body, chunk| {
                if let Some(chunk) = chunk.data_ref() {
                    body.extend_from_slice(chunk);
                }
                Ok(body)
            })
            .await
            .into_lua_err()?;

        // TODO: Decompress body if decompress is true and headers are present

        Ok(Self {
            inner: HyperRequest::from_parts(parts, Bytes::from(body)),
            redirects: 0,
            decompress,
        })
    }

    /**
        Returns the method of the request.
    */
    pub fn method(&self) -> Method {
        self.inner.method().clone()
    }

    /**
        Returns the path of the request.
    */
    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    /**
        Returns the query parameters of the request.
    */
    pub fn query(&self) -> HashMap<String, Vec<String>> {
        let uri = self.inner.uri();
        let url = uri.to_string().parse::<Url>().expect("uri is valid");

        let mut result = HashMap::<String, Vec<String>>::new();
        for (key, value) in url.query_pairs() {
            result
                .entry(key.into_owned())
                .or_default()
                .push(value.into_owned());
        }
        result
    }

    /**
        Returns the headers of the request.
    */
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /**
        Returns the body of the request.
    */
    pub fn body(&self) -> &[u8] {
        self.inner.body()
    }

    /**
        Returns the inner `hyper` request with its body
        type modified to `Full<Bytes>` for sending.
    */
    pub fn as_full(&self) -> HyperRequest<Full<Bytes>> {
        let mut builder = HyperRequest::builder()
            .version(self.inner.version())
            .method(self.inner.method())
            .uri(self.inner.uri());

        builder
            .headers_mut()
            .expect("request was valid")
            .extend(self.inner.headers().clone());

        let body = Full::new(self.inner.body().clone());
        builder.body(body).expect("request was valid")
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

impl LuaUserData for Request {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("method", |_, this| Ok(this.method().to_string()));
        fields.add_field_method_get("path", |_, this| Ok(this.path().to_string()));
        fields.add_field_method_get("query", |lua, this| {
            hash_map_to_table(lua, this.query(), false)
        });
        fields.add_field_method_get("headers", |lua, this| {
            header_map_to_table(lua, this.headers().clone(), this.decompress)
        });
        fields.add_field_method_get("body", |lua, this| lua.create_string(this.body()));
    }
}
