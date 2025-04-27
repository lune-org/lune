use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
};

use bstr::{BString, ByteSlice};
use hyper::{header::CONTENT_TYPE, StatusCode};
use mlua::prelude::*;

use crate::shared::headers::table_to_hash_map;

const DEFAULT_IP_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

const WEB_SOCKET_UPDGRADE_REQUEST_HANDLER: &str = r#"
return {
    status = 426,
    body = "Upgrade Required",
    headers = {
        Upgrade = "websocket",
    },
}
"#;

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

#[derive(Debug, Clone)]
pub struct ServeConfig {
    pub address: IpAddr,
    pub handle_request: LuaFunction,
    pub handle_web_socket: Option<LuaFunction>,
}

impl FromLua for ServeConfig {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::Function(f) = &value {
            // Single function = request handler, rest is default
            Ok(ServeConfig {
                handle_request: f.clone(),
                handle_web_socket: None,
                address: DEFAULT_IP_ADDRESS,
            })
        } else if let LuaValue::Table(t) = &value {
            // Table means custom options
            let address: Option<LuaString> = t.get("address")?;
            let handle_request: Option<LuaFunction> = t.get("handleRequest")?;
            let handle_web_socket: Option<LuaFunction> = t.get("handleWebSocket")?;
            if handle_request.is_some() || handle_web_socket.is_some() {
                let address: IpAddr = match &address {
                    Some(addr) => {
                        let addr_str = addr.to_str()?;

                        addr_str
                            .trim_start_matches("http://")
                            .trim_start_matches("https://")
                            .parse()
                            .map_err(|_e| LuaError::FromLuaConversionError {
                                from: value.type_name(),
                                to: "ServeConfig".to_string(),
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
                    to: "ServeConfig".to_string(),
                    message: Some(String::from(
                        "Invalid serve config - expected table with 'handleRequest' or 'handleWebSocket' function",
                    )),
                })
            }
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ServeConfig".to_string(),
                message: None,
            })
        }
    }
}
