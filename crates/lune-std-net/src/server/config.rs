use std::collections::HashMap;

use bstr::{BString, ByteSlice};
use hyper::{header::CONTENT_TYPE, StatusCode};
use mlua::prelude::*;

use crate::shared::headers::table_to_hash_map;

#[derive(Debug, Clone)]
pub struct ResponseConfig {
    pub status: StatusCode,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<Vec<u8>>,
}

impl FromLua for ResponseConfig {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        // If we just got a string we assume its a plaintext 200 response
        if let LuaValue::String(s) = value {
            Ok(Self {
                status: StatusCode::OK,
                headers: HashMap::from([(
                    CONTENT_TYPE.to_string(),
                    vec!["text/plain".to_string()],
                )]),
                body: Some(s.as_bytes().to_owned()),
            })
        } else if let LuaValue::Table(tab) = value {
            // If we got a table we are able to configure the entire response

            // Extract url
            let status = match tab.get::<u16>("status") {
                Ok(status) => Ok(StatusCode::from_u16(status).into_lua_err()?),
                Err(_) => Err(LuaError::runtime("Missing 'status' in response config")),
            }?;
            // Extract headers
            let headers = match tab.get::<LuaTable>("headers") {
                Ok(tab) => table_to_hash_map(tab, "headers")?,
                Err(_) => HashMap::new(),
            };
            // Extract body
            let body = match tab.get::<BString>("body") {
                Ok(config_body) => Some(config_body.as_bytes().to_owned()),
                Err(_) => None,
            };

            // All good, validated and we got what we need
            Ok(Self {
                status,
                headers,
                body,
            })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ResponseConfig".to_string(),
                message: Some(format!(
                    "Invalid response config - expected string or table, got {}",
                    value.type_name()
                )),
            })
        }
    }
}
