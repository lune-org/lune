use std::sync::atomic::{AtomicUsize, Ordering};

use hyper::{body::to_bytes, Body, Request};

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(super) struct ProcessedRequestId(usize);

impl ProcessedRequestId {
    pub fn new() -> Self {
        // NOTE: This may overflow after a couple billion requests,
        // but that's completely fine... unless a request is still
        // alive after billions more arrive and need to be handled
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub(super) struct ProcessedRequest {
    pub id: ProcessedRequestId,
    method: String,
    path: String,
    query: Vec<(String, String)>,
    headers: Vec<(String, Vec<u8>)>,
    body: Vec<u8>,
}

impl ProcessedRequest {
    pub async fn from_request(req: Request<Body>) -> LuaResult<Self> {
        let (head, body) = req.into_parts();

        // FUTURE: We can do extra processing like async decompression here
        let body = match to_bytes(body).await {
            Err(_) => return Err(LuaError::runtime("Failed to read request body bytes")),
            Ok(b) => b.to_vec(),
        };

        let method = head.method.to_string().to_ascii_uppercase();

        let mut path = head.uri.path().to_string();
        if path.is_empty() {
            path = "/".to_string();
        }

        let query = head
            .uri
            .query()
            .unwrap_or_default()
            .split('&')
            .filter_map(|q| q.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let mut headers = Vec::new();
        let mut header_name = String::new();
        for (name_opt, value) in head.headers.into_iter() {
            if let Some(name) = name_opt {
                header_name = name.to_string();
            }
            headers.push((header_name.clone(), value.as_bytes().to_vec()))
        }

        let id = ProcessedRequestId::new();

        Ok(Self {
            id,
            method,
            path,
            query,
            headers,
            body,
        })
    }

    pub fn into_lua_table(self, lua: &Lua) -> LuaResult<LuaTable> {
        // FUTURE: Make inner tables for query keys that have multiple values?
        let query = lua.create_table_with_capacity(0, self.query.len())?;
        for (key, value) in self.query.into_iter() {
            query.set(key, value)?;
        }

        let headers = lua.create_table_with_capacity(0, self.headers.len())?;
        for (key, value) in self.headers.into_iter() {
            headers.set(key, lua.create_string(value)?)?;
        }

        let body = lua.create_string(self.body)?;

        TableBuilder::new(lua)?
            .with_value("method", self.method)?
            .with_value("path", self.path)?
            .with_value("query", query)?
            .with_value("headers", headers)?
            .with_value("body", body)?
            .build_readonly()
    }
}
