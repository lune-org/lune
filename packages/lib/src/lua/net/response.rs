use std::collections::HashMap;

use hyper::{Body, Response};
use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum NetServeResponseKind {
    PlainText,
    Table,
}

#[derive(Debug, Clone)]
pub struct NetServeResponse {
    kind: NetServeResponseKind,
    status: u16,
    headers: HashMap<String, Vec<u8>>,
    body: Option<Vec<u8>>,
}

impl NetServeResponse {
    pub fn into_response(self) -> LuaResult<Response<Body>> {
        Ok(match self.kind {
            NetServeResponseKind::PlainText => Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body(Body::from(self.body.unwrap()))
                .map_err(LuaError::external)?,
            NetServeResponseKind::Table => {
                let mut response = Response::builder();
                for (key, value) in self.headers {
                    response = response.header(&key, value);
                }
                response
                    .status(self.status)
                    .body(Body::from(self.body.unwrap_or_default()))
                    .map_err(LuaError::external)?
            }
        })
    }
}

impl<'lua> FromLua<'lua> for NetServeResponse {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            // Plain strings from the handler are plaintext responses
            LuaValue::String(s) => Ok(Self {
                kind: NetServeResponseKind::PlainText,
                status: 200,
                headers: HashMap::new(),
                body: Some(s.as_bytes().to_vec()),
            }),
            // Tables are more detailed responses with potential status, headers, body
            LuaValue::Table(t) => {
                let status: Option<u16> = t.get("status")?;
                let headers: Option<LuaTable> = t.get("headers")?;
                let body: Option<LuaString> = t.get("body")?;

                let mut headers_map = HashMap::new();
                if let Some(headers) = headers {
                    for pair in headers.pairs::<String, LuaString>() {
                        let (h, v) = pair?;
                        headers_map.insert(h, v.as_bytes().to_vec());
                    }
                }

                let body_bytes = body.map(|s| s.as_bytes().to_vec());

                Ok(Self {
                    kind: NetServeResponseKind::Table,
                    status: status.unwrap_or(200),
                    headers: headers_map,
                    body: body_bytes,
                })
            }
            // Anything else is an error
            value => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "NetServeResponse",
                message: None,
            }),
        }
    }
}

impl<'lua> IntoLua<'lua> for NetServeResponse {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        if self.headers.len() > i32::MAX as usize {
            return Err(LuaError::ToLuaConversionError {
                from: "NetServeResponse",
                to: "table",
                message: Some("Too many header values".to_string()),
            });
        }
        let body = self.body.map(|b| lua.create_string(b)).transpose()?;
        let headers = lua.create_table_with_capacity(0, self.headers.len() as i32)?;
        for (key, value) in self.headers {
            headers.set(key, lua.create_string(&value)?)?;
        }
        let table = lua.create_table_with_capacity(0, 3)?;
        table.set("status", self.status)?;
        table.set("headers", headers)?;
        table.set("body", body)?;
        table.set_readonly(true);
        Ok(LuaValue::Table(table))
    }
}
