use std::{collections::HashMap, net::SocketAddr};

use http::request::Parts;

use mlua::prelude::*;

use lune_utils::TableBuilder;

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

        let query: HashMap<LuaString, LuaString> = self
            .head
            .uri
            .query()
            .unwrap_or_default()
            .split('&')
            .filter_map(|q| q.split_once('='))
            .map(|(k, v)| {
                let k = lua.create_string(k)?;
                let v = lua.create_string(v)?;
                Ok((k, v))
            })
            .collect::<LuaResult<_>>()?;

        let headers: HashMap<LuaString, LuaString> = self
            .head
            .headers
            .iter()
            .map(|(k, v)| {
                let k = lua.create_string(k.as_str())?;
                let v = lua.create_string(v.as_bytes())?;
                Ok((k, v))
            })
            .collect::<LuaResult<_>>()?;

        TableBuilder::new(lua)?
            .with_value("method", method)?
            .with_value("path", path)?
            .with_value("query", query)?
            .with_value("headers", headers)?
            .with_value("body", body)?
            .build()
    }
}
