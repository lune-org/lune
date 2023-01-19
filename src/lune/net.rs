use std::{collections::HashMap, str::FromStr};

use mlua::{Error, Lua, LuaSerdeExt, Result, UserData, UserDataMethods, Value};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};

use crate::utils::get_github_user_agent_header;

pub struct LuneNet();

impl LuneNet {
    pub fn new() -> Self {
        Self()
    }
}

impl UserData for LuneNet {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("jsonEncode", net_json_encode);
        methods.add_function("jsonDecode", net_json_decode);
        methods.add_async_function("request", net_request);
    }
}

fn net_json_encode(_: &Lua, (val, pretty): (Value, Option<bool>)) -> Result<String> {
    if let Some(true) = pretty {
        serde_json::to_string_pretty(&val).map_err(Error::external)
    } else {
        serde_json::to_string(&val).map_err(Error::external)
    }
}

fn net_json_decode(lua: &Lua, json: String) -> Result<Value> {
    let json: serde_json::Value = serde_json::from_str(&json).map_err(Error::external)?;
    lua.to_value(&json)
}

async fn net_request<'lua>(lua: &'lua Lua, config: Value<'lua>) -> Result<Value<'lua>> {
    // Extract stuff from config and make sure its all valid
    let (url, method, headers, body) = match config {
        Value::String(s) => {
            let url = s.to_string_lossy().to_string();
            let method = "GET".to_string();
            (url, method, None, None)
        }
        Value::Table(tab) => {
            // Extract url
            let url = match tab.raw_get::<&str, mlua::String>("url") {
                Ok(config_url) => config_url.to_string_lossy().to_string(),
                Err(_) => return Err(Error::RuntimeError("Missing 'url' in config".to_string())),
            };
            // Extract method
            let method = match tab.raw_get::<&str, mlua::String>("method") {
                Ok(config_method) => config_method.to_string_lossy().trim().to_ascii_uppercase(),
                Err(_) => "GET".to_string(),
            };
            // Extract headers
            let headers = match tab.raw_get::<&str, mlua::Table>("headers") {
                Ok(config_headers) => {
                    let mut lua_headers = HeaderMap::new();
                    for pair in config_headers.pairs::<mlua::String, mlua::String>() {
                        let (key, value) = pair?;
                        lua_headers.insert(
                            HeaderName::from_str(key.to_str()?).map_err(Error::external)?,
                            HeaderValue::from_str(value.to_str()?).map_err(Error::external)?,
                        );
                    }
                    Some(lua_headers)
                }
                Err(_) => None,
            };
            // Extract body
            let body = match tab.raw_get::<&str, mlua::String>("body") {
                Ok(config_body) => Some(config_body.as_bytes().to_owned()),
                Err(_) => None,
            };
            (url, method, headers, body)
        }
        _ => return Err(Error::RuntimeError("Invalid config value".to_string())),
    };
    // Convert method string into proper enum
    let method = match Method::from_str(&method) {
        Ok(meth) => meth,
        Err(_) => {
            return Err(Error::RuntimeError(format!(
                "Invalid config method '{}'",
                &method
            )))
        }
    };
    // Extract headers from config, force user agent
    let mut header_map = if let Some(headers) = headers {
        headers
    } else {
        HeaderMap::new()
    };
    header_map.insert(
        "User-Agent",
        HeaderValue::from_str(&get_github_user_agent_header()).map_err(Error::external)?,
    );
    // Create a client to send a request with
    // FUTURE: Try to reuse this client
    let client = reqwest::Client::builder()
        .build()
        .map_err(Error::external)?;
    // Create and send the request
    let mut request = client.request(method, url).headers(header_map);
    if let Some(body) = body {
        request = request.body(body)
    }
    let response = request.send().await.map_err(Error::external)?;
    // Extract status, headers, body
    let res_status = response.status();
    let res_headers = response.headers().to_owned();
    let res_bytes = response.bytes().await.map_err(Error::external)?;
    // Construct and return a readonly lua table with results
    let tab = lua.create_table()?;
    tab.raw_set("ok", res_status.is_success())?;
    tab.raw_set("statusCode", res_status.as_u16())?;
    tab.raw_set(
        "statusMessage",
        res_status.canonical_reason().unwrap_or("?"),
    )?;
    tab.raw_set(
        "headers",
        res_headers
            .iter()
            .filter(|(_, value)| value.to_str().is_ok())
            .map(|(key, value)| (key.as_str(), value.to_str().unwrap()))
            .collect::<HashMap<_, _>>(),
    )?;
    tab.raw_set("body", lua.create_string(&res_bytes)?)?;
    tab.set_readonly(true);
    Ok(Value::Table(tab))
}
