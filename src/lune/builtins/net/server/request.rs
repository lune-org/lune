use std::{collections::HashMap, net::SocketAddr};

use http::request::Parts;

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

pub(super) struct LuaRequest {
    pub(super) _remote_addr: SocketAddr,
    pub(super) head: Parts,
    pub(super) body: Vec<u8>,
}

impl LuaRequest {
    pub fn into_lua_table(self, lua: &Lua) -> LuaResult<LuaTable> {
        let method = self.head.method.as_str().to_string();
        let path = self.head.uri.path().to_string();
        let body = lua.create_string(&self.body)?;

        let query: HashMap<String, String> = self
            .head
            .uri
            .query()
            .unwrap_or_default()
            .split('&')
            .filter_map(|q| q.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let headers: HashMap<String, Vec<u8>> = self
            .head
            .headers
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.as_bytes().to_vec()))
            .collect();

        TableBuilder::new(lua)?
            .with_value("method", method)?
            .with_value("path", path)?
            .with_value("query", query)?
            .with_value("headers", headers)?
            .with_value("body", body)?
            .build()
    }
}
