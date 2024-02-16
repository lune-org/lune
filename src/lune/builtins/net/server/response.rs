use std::str::FromStr;

use http_body_util::Full;
use hyper::{
    body::Bytes,
    header::{HeaderName, HeaderValue},
    HeaderMap, Response,
};

use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) enum LuaResponseKind {
    PlainText,
    Table,
}

pub(super) struct LuaResponse {
    pub(super) kind: LuaResponseKind,
    pub(super) status: u16,
    pub(super) headers: HeaderMap,
    pub(super) body: Option<Vec<u8>>,
}

impl LuaResponse {
    pub(super) fn into_response(self) -> LuaResult<Response<Full<Bytes>>> {
        Ok(match self.kind {
            LuaResponseKind::PlainText => Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body(Full::new(Bytes::from(self.body.unwrap())))
                .into_lua_err()?,
            LuaResponseKind::Table => {
                let mut response = Response::builder()
                    .status(self.status)
                    .body(Full::new(Bytes::from(self.body.unwrap_or_default())))
                    .into_lua_err()?;
                response.headers_mut().extend(self.headers);
                response
            }
        })
    }
}

impl FromLua<'_> for LuaResponse {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            // Plain strings from the handler are plaintext responses
            LuaValue::String(s) => Ok(Self {
                kind: LuaResponseKind::PlainText,
                status: 200,
                headers: HeaderMap::new(),
                body: Some(s.as_bytes().to_vec()),
            }),
            // Tables are more detailed responses with potential status, headers, body
            LuaValue::Table(t) => {
                let status: Option<u16> = t.get("status")?;
                let headers: Option<LuaTable> = t.get("headers")?;
                let body: Option<LuaString> = t.get("body")?;

                let mut headers_map = HeaderMap::new();
                if let Some(headers) = headers {
                    for pair in headers.pairs::<String, LuaString>() {
                        let (h, v) = pair?;
                        let name = HeaderName::from_str(&h).into_lua_err()?;
                        let value = HeaderValue::from_bytes(v.as_bytes()).into_lua_err()?;
                        headers_map.insert(name, value);
                    }
                }

                let body_bytes = body.map(|s| s.as_bytes().to_vec());

                Ok(Self {
                    kind: LuaResponseKind::Table,
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
