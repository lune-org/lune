use std::collections::HashMap;

use mlua::prelude::*;

use reqwest::Method;

use super::util::table_to_hash_map;

// Net request config

#[derive(Debug, Clone)]
pub struct RequestConfigOptions {
    pub decompress: bool,
}

impl Default for RequestConfigOptions {
    fn default() -> Self {
        Self { decompress: true }
    }
}

impl<'lua> FromLua<'lua> for RequestConfigOptions {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        // Nil means default options, table means custom options
        if let LuaValue::Nil = value {
            return Ok(Self::default());
        } else if let LuaValue::Table(tab) = value {
            // Extract flags
            let decompress = match tab.get::<_, Option<bool>>("decompress") {
                Ok(decomp) => Ok(decomp.unwrap_or(true)),
                Err(_) => Err(LuaError::RuntimeError(
                    "Invalid option value for 'decompress' in request config options".to_string(),
                )),
            }?;
            return Ok(Self { decompress });
        }
        // Anything else is invalid
        Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "RequestConfigOptions",
            message: Some(format!(
                "Invalid request config options - expected table or nil, got {}",
                value.type_name()
            )),
        })
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

impl FromLua<'_> for RequestConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        // If we just got a string we assume its a GET request to a given url
        if let LuaValue::String(s) = value {
            return Ok(Self {
                url: s.to_string_lossy().to_string(),
                method: Method::GET,
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                options: Default::default(),
            });
        }
        // If we got a table we are able to configure the entire request
        if let LuaValue::Table(tab) = value {
            // Extract url
            let url = match tab.get::<_, LuaString>("url") {
                Ok(config_url) => Ok(config_url.to_string_lossy().to_string()),
                Err(_) => Err(LuaError::runtime("Missing 'url' in request config")),
            }?;
            // Extract method
            let method = match tab.get::<_, LuaString>("method") {
                Ok(config_method) => config_method.to_string_lossy().trim().to_ascii_uppercase(),
                Err(_) => "GET".to_string(),
            };
            // Extract query
            let query = match tab.get::<_, LuaTable>("query") {
                Ok(tab) => table_to_hash_map(tab, "query")?,
                Err(_) => HashMap::new(),
            };
            // Extract headers
            let headers = match tab.get::<_, LuaTable>("headers") {
                Ok(tab) => table_to_hash_map(tab, "headers")?,
                Err(_) => HashMap::new(),
            };
            // Extract body
            let body = match tab.get::<_, LuaString>("body") {
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
            let options = match tab.get::<_, LuaValue>("options") {
                Ok(opts) => RequestConfigOptions::from_lua(opts, lua)?,
                Err(_) => RequestConfigOptions::default(),
            };
            // All good, validated and we got what we need
            return Ok(Self {
                url,
                method,
                query,
                headers,
                body,
                options,
            });
        };
        // Anything else is invalid
        Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "RequestConfig",
            message: Some(format!(
                "Invalid request config - expected string or table, got {}",
                value.type_name()
            )),
        })
    }
}

// Net serve config

#[derive(Debug)]
pub struct ServeConfig<'a> {
    pub handle_request: LuaFunction<'a>,
    pub handle_web_socket: Option<LuaFunction<'a>>,
    pub address: Option<LuaString<'a>>,
}

impl<'lua> FromLua<'lua> for ServeConfig<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let message = match &value {
            LuaValue::Function(f) => {
                return Ok(ServeConfig {
                    handle_request: f.clone(),
                    handle_web_socket: None,
                    address: None,
                })
            }
            LuaValue::Table(t) => {
                let handle_request: Option<LuaFunction> = t.get("handleRequest")?;
                let handle_web_socket: Option<LuaFunction> = t.get("handleWebSocket")?;
                let address: Option<LuaString> = t.get("address")?;
                if handle_request.is_some() || handle_web_socket.is_some() {
                    return Ok(ServeConfig {
                        handle_request: handle_request.unwrap_or_else(|| {
                            let chunk = r#"
                            return {
                                status = 426,
                                body = "Upgrade Required",
                                headers = {
                                    Upgrade = "websocket",
                                },
                            }
                            "#;
                            lua.load(chunk)
                                .into_function()
                                .expect("Failed to create default http responder function")
                        }),
                        handle_web_socket,
                        address,
                    });
                } else {
                    Some("Missing handleRequest and / or handleWebSocket".to_string())
                }
            }
            _ => None,
        };
        Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "ServeConfig",
            message,
        })
    }
}
