use std::str::FromStr;

use mlua::prelude::*;

use hyper::{header::HeaderName, http::HeaderValue, HeaderMap};
use reqwest::{IntoUrl, Method, RequestBuilder};

const REGISTRY_KEY: &str = "NetClient";

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

    pub fn into_registry(self, lua: &Lua) {
        lua.set_named_registry_value(REGISTRY_KEY, self)
            .expect("Failed to store NetClient in lua registry");
    }

    pub fn from_registry(lua: &Lua) -> Self {
        lua.named_registry_value(REGISTRY_KEY)
            .expect("Failed to get NetClient from lua registry")
    }
}

impl LuaUserData for NetClient {}

impl<'lua> FromLua<'lua> for NetClient {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(ctx) = ud.borrow::<NetClient>() {
                return Ok(ctx.clone());
            }
        }
        unreachable!("NetClient should only be used from registry")
    }
}

impl<'lua> From<&'lua Lua> for NetClient {
    fn from(value: &'lua Lua) -> Self {
        value
            .named_registry_value(REGISTRY_KEY)
            .expect("Missing require context in lua registry")
    }
}
