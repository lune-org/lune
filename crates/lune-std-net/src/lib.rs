#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

mod client;
mod config;
mod server;
mod util;
mod websocket;

use lune_utils::TableBuilder;

use self::{
    client::{NetClient, NetClientBuilder},
    config::{RequestConfig, ServeConfig},
    server::serve,
    util::create_user_agent_header,
    websocket::NetWebSocket,
};

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `net` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `net` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    NetClientBuilder::new()
        .headers(&[("User-Agent", create_user_agent_header(&lua)?)])?
        .build()?
        .into_registry(&lua);
    TableBuilder::new(lua)?
        .with_async_function("request", net_request)?
        .with_async_function("socket", net_socket)?
        .with_async_function("serve", net_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}

async fn net_request(lua: Lua, config: RequestConfig) -> LuaResult<LuaTable> {
    let client = NetClient::from_registry(&lua);
    // NOTE: We spawn the request as a background task to free up resources in lua
    let res = lua.spawn(async move { client.request(config).await });
    res.await?.into_lua_table(&lua)
}

async fn net_socket(lua: Lua, url: String) -> LuaResult<LuaValue> {
    let (ws, _) = tokio_tungstenite::connect_async(url).await.into_lua_err()?;
    NetWebSocket::new(ws).into_lua(&lua)
}

async fn net_serve(lua: Lua, (port, config): (u16, ServeConfig)) -> LuaResult<LuaTable> {
    serve(lua, port, config).await
}

fn net_url_encode(
    lua: &Lua,
    (lua_string, as_binary): (LuaString, Option<bool>),
) -> LuaResult<LuaValue> {
    if matches!(as_binary, Some(true)) {
        urlencoding::encode_binary(&lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::encode(&lua_string.to_str()?).into_lua(lua)
    }
}

fn net_url_decode(
    lua: &Lua,
    (lua_string, as_binary): (LuaString, Option<bool>),
) -> LuaResult<LuaValue> {
    if matches!(as_binary, Some(true)) {
        urlencoding::decode_binary(&lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::decode(&lua_string.to_str()?)
            .map_err(|e| LuaError::RuntimeError(format!("Encountered invalid encoding - {e}")))?
            .into_lua(lua)
    }
}
