use std::collections::HashMap;

use mlua::prelude::*;

use reqwest::Method;

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
            let decompress = match tab.raw_get::<_, Option<bool>>("decompress") {
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
pub struct RequestConfig<'a> {
    pub url: String,
    pub method: Method,
    pub query: HashMap<LuaString<'a>, LuaString<'a>>,
    pub headers: HashMap<LuaString<'a>, LuaString<'a>>,
    pub body: Option<Vec<u8>>,
    pub options: RequestConfigOptions,
}

impl<'lua> FromLua<'lua> for RequestConfig<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
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
            let url = match tab.raw_get::<_, LuaString>("url") {
                Ok(config_url) => Ok(config_url.to_string_lossy().to_string()),
                Err(_) => Err(LuaError::RuntimeError(
                    "Missing 'url' in request config".to_string(),
                )),
            }?;
            // Extract method
            let method = match tab.raw_get::<_, LuaString>("method") {
                Ok(config_method) => config_method.to_string_lossy().trim().to_ascii_uppercase(),
                Err(_) => "GET".to_string(),
            };
            // Extract query
            let query = match tab.raw_get::<_, LuaTable>("query") {
                Ok(config_headers) => {
                    let mut lua_headers = HashMap::new();
                    for pair in config_headers.pairs::<LuaString, LuaString>() {
                        let (key, value) = pair?.to_owned();
                        lua_headers.insert(key, value);
                    }
                    lua_headers
                }
                Err(_) => HashMap::new(),
            };
            // Extract headers
            let headers = match tab.raw_get::<_, LuaTable>("headers") {
                Ok(config_headers) => {
                    let mut lua_headers = HashMap::new();
                    for pair in config_headers.pairs::<LuaString, LuaString>() {
                        let (key, value) = pair?.to_owned();
                        lua_headers.insert(key, value);
                    }
                    lua_headers
                }
                Err(_) => HashMap::new(),
            };
            // Extract body
            let body = match tab.raw_get::<_, LuaString>("body") {
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
            let options = match tab.raw_get::<_, LuaValue>("options") {
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

pub struct ServeConfig<'a> {
    pub handle_request: LuaFunction<'a>,
    pub handle_web_socket: Option<LuaFunction<'a>>,
}

impl<'lua> FromLua<'lua> for ServeConfig<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let message = match &value {
            LuaValue::Function(f) => {
                return Ok(ServeConfig {
                    handle_request: f.clone(),
                    handle_web_socket: None,
                })
            }
            LuaValue::Table(t) => {
                let handle_request: Option<LuaFunction> = t.raw_get("handleRequest")?;
                let handle_web_socket: Option<LuaFunction> = t.raw_get("handleWebSocket")?;
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
