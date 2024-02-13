use std::{collections::HashMap, net::Ipv4Addr};

use mlua::prelude::*;

use reqwest::Method;

use super::util::table_to_hash_map;

const DEFAULT_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

const WEB_SOCKET_UPDGRADE_REQUEST_HANDLER: &str = r#"
return {
    status = 426,
    body = "Upgrade Required",
    headers = {
        Upgrade = "websocket",
    },
}
"#;

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
        if let LuaValue::Nil = value {
            // Nil means default options
            Ok(Self::default())
        } else if let LuaValue::Table(tab) = value {
            // Table means custom options
            let decompress = match tab.get::<_, Option<bool>>("decompress") {
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
                to: "RequestConfigOptions",
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

impl FromLua<'_> for RequestConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        // If we just got a string we assume its a GET request to a given url
        if let LuaValue::String(s) = value {
            Ok(Self {
                url: s.to_string_lossy().to_string(),
                method: Method::GET,
                query: HashMap::new(),
                headers: HashMap::new(),
                body: None,
                options: Default::default(),
            })
        } else if let LuaValue::Table(tab) = value {
            // If we got a table we are able to configure the entire request
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
                to: "RequestConfig",
                message: Some(format!(
                    "Invalid request config - expected string or table, got {}",
                    value.type_name()
                )),
            })
        }
    }
}

// Net serve config

#[derive(Debug)]
pub struct ServeConfig<'a> {
    pub address: Ipv4Addr,
    pub handle_request: LuaFunction<'a>,
    pub handle_web_socket: Option<LuaFunction<'a>>,
}

impl<'lua> FromLua<'lua> for ServeConfig<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Function(f) = &value {
            // Single function = request handler, rest is default
            Ok(ServeConfig {
                handle_request: f.clone(),
                handle_web_socket: None,
                address: DEFAULT_IP_ADDRESS.clone(),
            })
        } else if let LuaValue::Table(t) = &value {
            // Table means custom options
            let address: Option<LuaString> = t.get("address")?;
            let handle_request: Option<LuaFunction> = t.get("handleRequest")?;
            let handle_web_socket: Option<LuaFunction> = t.get("handleWebSocket")?;
            if handle_request.is_some() || handle_web_socket.is_some() {
                let address: Ipv4Addr = match &address {
                    Some(addr) => {
                        let addr_str = addr.to_str()?;

                        addr_str
                            .trim_start_matches("http://")
                            .trim_start_matches("https://")
                            .parse()
                            .map_err(|_e| LuaError::FromLuaConversionError {
                                from: value.type_name(),
                                to: "ServeConfig",
                                message: Some(format!(
                                    "IP address format is incorrect - \
                                    expected an IP in the form 'http://0.0.0.0' or '0.0.0.0', \
                                    got '{addr_str}'"
                                )),
                            })?
                    }
                    None => DEFAULT_IP_ADDRESS,
                };

                Ok(Self {
                    address,
                    handle_request: handle_request.unwrap_or_else(|| {
                        lua.load(WEB_SOCKET_UPDGRADE_REQUEST_HANDLER)
                            .into_function()
                            .expect("Failed to create default http responder function")
                    }),
                    handle_web_socket,
                })
            } else {
                Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "ServeConfig",
                    message: Some(String::from(
                        "Invalid serve config - expected table with 'handleRequest' or 'handleWebSocket' function",
                    )),
                })
            }
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ServeConfig",
                message: None,
            })
        }
    }
}
