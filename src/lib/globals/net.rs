use std::collections::HashMap;

use mlua::prelude::*;
use reqwest::Method;

use crate::utils::{net::get_request_user_agent_header, table::TableBuilder};

pub fn create(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set(
        "net",
        TableBuilder::new(lua)?
            .with_function("jsonEncode", net_json_encode)?
            .with_function("jsonDecode", net_json_decode)?
            .with_async_function("request", net_request)?
            .build_readonly()?,
    )
}

fn net_json_encode(_: &Lua, (val, pretty): (LuaValue, Option<bool>)) -> LuaResult<String> {
    if let Some(true) = pretty {
        serde_json::to_string_pretty(&val).map_err(LuaError::external)
    } else {
        serde_json::to_string(&val).map_err(LuaError::external)
    }
}

fn net_json_decode(lua: &Lua, json: String) -> LuaResult<LuaValue> {
    let json: serde_json::Value = serde_json::from_str(&json).map_err(LuaError::external)?;
    lua.to_value(&json)
}

async fn net_request<'lua>(lua: &'lua Lua, config: LuaValue<'lua>) -> LuaResult<LuaTable<'lua>> {
    // Extract stuff from config and make sure its all valid
    let (url, method, headers, body) = match config {
        LuaValue::String(s) => {
            let url = s.to_string_lossy().to_string();
            let method = "GET".to_string();
            Ok((url, method, HashMap::new(), None))
        }
        LuaValue::Table(tab) => {
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
            Ok((url, method, headers, body))
        }
        value => Err(LuaError::RuntimeError(format!(
            "Invalid request config - expected string or table, got {}",
            value.type_name()
        ))),
    }?;
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
    // TODO: Figure out how to reuse this client
    let client = reqwest::ClientBuilder::new()
        .build()
        .map_err(LuaError::external)?;
    // Create and send the request
    let mut request = client.request(method, &url);
    for (header, value) in headers {
        request = request.header(header.to_str()?, value.to_str()?);
    }
    let res = request
        .header("User-Agent", &get_request_user_agent_header()) // Always force user agent
        .body(body.unwrap_or_default())
        .send()
        .await
        .map_err(LuaError::external)?;
    // Extract status, headers
    let res_status = res.status().as_u16();
    let res_status_text = res.status().canonical_reason();
    let res_headers = res
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_owned()))
        .collect::<HashMap<String, String>>();
    // Read response bytes
    let res_bytes = res.bytes().await.map_err(LuaError::external)?;
    // Construct and return a readonly lua table with results
    TableBuilder::new(lua)?
        .with_value("ok", (200..300).contains(&res_status))?
        .with_value("statusCode", res_status)?
        .with_value("statusMessage", res_status_text)?
        .with_value("headers", res_headers)?
        .with_value("body", lua.create_string(&res_bytes)?)?
        .build_readonly()
}
