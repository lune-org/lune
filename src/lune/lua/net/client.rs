use std::str::FromStr;

use mlua::prelude::*;

use hyper::{header::HeaderName, http::HeaderValue, HeaderMap};
use reqwest::{IntoUrl, Method, RequestBuilder};

pub struct NetClientBuilder {
    builder: reqwest::ClientBuilder,
}

impl NetClientBuilder {
    pub fn new() -> NetClientBuilder {
        Self {
            builder: reqwest::ClientBuilder::new(),
        }
    }

    pub fn headers<K, V>(mut self, headers: &[(K, V)]) -> LuaResult<Self>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let mut map = HeaderMap::new();
        for (key, val) in headers {
            let hkey = HeaderName::from_str(key.as_ref()).into_lua_err()?;
            let hval = HeaderValue::from_bytes(val.as_ref()).into_lua_err()?;
            map.insert(hkey, hval);
        }
        self.builder = self.builder.default_headers(map);
        Ok(self)
    }

    pub fn build(self) -> LuaResult<NetClient> {
        let client = self.builder.build().into_lua_err()?;
        Ok(NetClient(client))
    }
}

#[derive(Debug, Clone)]
pub struct NetClient(reqwest::Client);

impl NetClient {
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.0.request(method, url)
    }
}

impl LuaUserData for NetClient {}
