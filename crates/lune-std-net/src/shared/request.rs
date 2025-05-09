use std::{collections::HashMap, net::SocketAddr};

use url::Url;

use hyper::{body::Incoming, HeaderMap, Method, Request as HyperRequest};

use mlua::prelude::*;

use crate::{
    body::{handle_incoming_body, ReadableBody},
    shared::{
        headers::{hash_map_to_table, header_map_to_table},
        lua::{lua_table_to_header_map, lua_value_to_method},
    },
};

#[derive(Debug, Clone)]
pub struct RequestOptions {
    pub decompress: bool,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self { decompress: true }
    }
}

impl FromLua for RequestOptions {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            // Nil means default options
            Ok(Self::default())
        } else if let LuaValue::Table(tab) = value {
            // Table means custom options
            let decompress = match tab.get::<Option<bool>>("decompress") {
                Ok(decomp) => Ok(decomp.unwrap_or(true)),
                Err(_) => Err(LuaError::RuntimeError(
                    "Invalid option value for 'decompress' in request options".to_string(),
                )),
            }?;
            Ok(Self { decompress })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "RequestOptions".to_string(),
                message: Some(format!(
                    "Invalid request options - expected table or nil, got {}",
                    value.type_name()
                )),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub(crate) inner: HyperRequest<ReadableBody>,
    pub(crate) address: Option<SocketAddr>,
    pub(crate) redirects: Option<usize>,
    pub(crate) decompress: bool,
}

impl Request {
    /**
        Creates a new request from a raw incoming request.
    */
    pub async fn from_incoming(
        incoming: HyperRequest<Incoming>,
        decompress: bool,
    ) -> LuaResult<Self> {
        let (parts, body) = incoming.into_parts();

        let (body, decompress) = handle_incoming_body(&parts.headers, body, decompress).await?;

        Ok(Self {
            inner: HyperRequest::from_parts(parts, ReadableBody::from(body)),
            address: None,
            redirects: None,
            decompress,
        })
    }

    /**
        Attaches a socket address to the request.

        This will make the `ip` and `port` fields available on the request.
    */
    pub fn with_address(mut self, address: SocketAddr) -> Self {
        self.address = Some(address);
        self
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

        let mut result = HashMap::<String, Vec<String>>::new();

        if let Some(query) = uri.query() {
            for (key, value) in form_urlencoded::parse(query.as_bytes()) {
                result
                    .entry(key.to_string())
                    .or_default()
                    .push(value.to_string());
            }
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
        self.inner.body().as_slice()
    }

    /**
        Clones the inner `hyper` request.
    */
    #[allow(dead_code)]
    pub fn clone_inner(&self) -> HyperRequest<ReadableBody> {
        self.inner.clone()
    }

    /**
        Takes the inner `hyper` request by ownership.
    */
    #[allow(dead_code)]
    pub fn into_inner(self) -> HyperRequest<ReadableBody> {
        self.inner
    }
}

impl<B: Into<ReadableBody>> From<HyperRequest<B>> for Request {
    fn from(inner: HyperRequest<B>) -> Self {
        let (parts, body) = inner.into_parts();
        Self {
            inner: HyperRequest::from_parts(parts, body.into()),
            address: None,
            redirects: None,
            decompress: false,
        }
    }
}

impl FromLua for Request {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = value {
            // If we just got a string we assume
            // its a GET request to a given url
            let uri = s.to_str()?;
            let uri = uri.parse().into_lua_err()?;

            let mut request = HyperRequest::new(ReadableBody::empty());
            *request.uri_mut() = uri;

            Ok(Self {
                inner: request,
                address: None,
                redirects: None,
                decompress: RequestOptions::default().decompress,
            })
        } else if let LuaValue::Table(tab) = value {
            // If we got a table we are able to configure the
            // entire request, maybe with extra options too
            let options = match tab.get::<LuaValue>("options") {
                Ok(opts) => RequestOptions::from_lua(opts, lua)?,
                Err(_) => RequestOptions::default(),
            };

            // Extract url (required) + optional structured query params
            let url = tab.get::<LuaString>("url")?;
            let mut url = url.to_str()?.parse::<Url>().into_lua_err()?;
            if let Some(t) = tab.get::<Option<LuaTable>>("query")? {
                let mut query = url.query_pairs_mut();
                for pair in t.pairs::<LuaString, LuaString>() {
                    let (key, value) = pair?;
                    let key = key.to_str()?;
                    let value = value.to_str()?;
                    query.append_pair(&key, &value);
                }
            }

            // Extract method
            let method = tab.get::<LuaValue>("method")?;
            let method = lua_value_to_method(&method)?;

            // Extract headers
            let headers = tab.get::<Option<LuaTable>>("headers")?;
            let headers = headers
                .map(|t| lua_table_to_header_map(&t))
                .transpose()?
                .unwrap_or_default();

            // Extract body
            let body = tab.get::<ReadableBody>("body")?;

            // Build the full request
            let mut request = HyperRequest::new(body);
            request.headers_mut().extend(headers);
            *request.uri_mut() = url.to_string().parse().unwrap();
            *request.method_mut() = method;

            // All good, validated and we got what we need
            Ok(Self {
                inner: request,
                address: None,
                redirects: None,
                decompress: options.decompress,
            })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Request".to_string(),
                message: Some(format!(
                    "Invalid request - expected string or table, got {}",
                    value.type_name()
                )),
            })
        }
    }
}

impl LuaUserData for Request {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ip", |_, this| {
            Ok(this.address.map(|address| address.ip().to_string()))
        });
        fields.add_field_method_get("port", |_, this| {
            Ok(this.address.map(|address| address.port()))
        });
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
