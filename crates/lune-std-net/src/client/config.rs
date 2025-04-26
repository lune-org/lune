use std::collections::HashMap;

use bstr::{BString, ByteSlice};
use hyper::Method;
use mlua::prelude::*;

use crate::shared::headers::table_to_hash_map;

#[derive(Debug, Clone)]
pub struct RequestConfigOptions {
    pub decompress: bool,
}

impl Default for RequestConfigOptions {
    fn default() -> Self {
        Self { decompress: true }
    }
}

impl FromLua for RequestConfigOptions {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            // Nil means default options
            Ok(Self::default())
        } else if let LuaValue::Table(tab) = value {
            // Table means custom options
            let decompress = match tab.get::<Option<bool>>("decompress") {
                Ok(decomp) => Ok(decomp.unwrap_or(true)),
                Err(_) => Err(LuaError::RuntimeError(
                    "Invalid option value for 'decompress' in request config options".to_string(),
                )),
            }?;
            Ok(Self { decompress })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "RequestConfigOptions".to_string(),
                message: Some(format!(
                    "Invalid request config options - expected table or nil, got {}",
                    value.type_name()
                )),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub url: String,
    pub method: Method,
    pub query: HashMap<String, Vec<String>>,
    pub headers: HashMap<String, Vec<String>>,
    pub body: Option<Vec<u8>>,
    pub options: RequestConfigOptions,
}

impl FromLua for RequestConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        // If we just got a string we assume its a GET request to a given url
        if let LuaValue::String(s) = value {
            Ok(Self {
                url: s.to_string_lossy().to_string(),
                method: Method::GET,
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                options: RequestConfigOptions::default(),
            })
        } else if let LuaValue::Table(tab) = value {
            // If we got a table we are able to configure the entire request

            // Extract url
            let url = match tab.get::<LuaString>("url") {
                Ok(config_url) => Ok(config_url.to_string_lossy().to_string()),
                Err(_) => Err(LuaError::runtime("Missing 'url' in request config")),
            }?;
            // Extract method
            let method = match tab.get::<LuaString>("method") {
                Ok(config_method) => config_method.to_string_lossy().trim().to_ascii_uppercase(),
                Err(_) => "GET".to_string(),
            };
            // Extract query
            let query = match tab.get::<LuaTable>("query") {
                Ok(tab) => table_to_hash_map(tab, "query")?,
                Err(_) => HashMap::new(),
            };
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

            // Convert method string into proper enum
            let method = method.trim().to_ascii_uppercase();
            let method = match method.as_ref() {
                "GET" => Ok(Method::GET),
                "POST" => Ok(Method::POST),
                "PUT" => Ok(Method::PUT),
                "DELETE" => Ok(Method::DELETE),
                "HEAD" => Ok(Method::HEAD),
                "OPTIONS" => Ok(Method::OPTIONS),
                "PATCH" => Ok(Method::PATCH),
                _ => Err(LuaError::RuntimeError(format!(
                    "Invalid request config method '{}'",
                    &method
                ))),
            }?;

            // Parse any extra options given
            let options = match tab.get::<LuaValue>("options") {
                Ok(opts) => RequestConfigOptions::from_lua(opts, lua)?,
                Err(_) => RequestConfigOptions::default(),
            };

            // All good, validated and we got what we need
            Ok(Self {
                url,
                method,
                query,
                headers,
                body,
                options,
            })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "RequestConfig".to_string(),
                message: Some(format!(
                    "Invalid request config - expected string or table, got {}",
                    value.type_name()
                )),
            })
        }
    }
}
