use std::{collections::HashMap, net::SocketAddr};

use http::request::Parts;

use mlua::prelude::*;

pub(super) struct LuaRequest {
    pub(super) _remote_addr: SocketAddr,
    pub(super) head: Parts,
    pub(super) body: Vec<u8>,
}

impl LuaUserData for LuaRequest {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("method", |_, this| {
            Ok(this.head.method.as_str().to_string())
        });

        fields.add_field_method_get("path", |_, this| Ok(this.head.uri.path().to_string()));

        fields.add_field_method_get("query", |_, this| {
            let query: HashMap<String, String> = this
                .head
                .uri
                .query()
                .unwrap_or_default()
                .split('&')
                .filter_map(|q| q.split_once('='))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            Ok(query)
        });

        fields.add_field_method_get("headers", |_, this| {
            let headers: HashMap<String, Vec<u8>> = this
                .head
                .headers
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), v.as_bytes().to_vec()))
                .collect();
            Ok(headers)
        });

        fields.add_field_method_get("body", |lua, this| lua.create_string(&this.body));
    }
}
