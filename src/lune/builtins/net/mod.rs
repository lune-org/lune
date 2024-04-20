#![allow(unused_variables)]

use bstr::BString;
use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

mod client;
mod config;
mod server;
mod util;
mod websocket;

use crate::lune::util::TableBuilder;

use self::{
    client::{NetClient, NetClientBuilder},
    config::{RequestConfig, ServeConfig},
    server::serve,
    util::create_user_agent_header,
    websocket::NetWebSocket,
};

use super::serde::encode_decode::{EncodeDecodeConfig, EncodeDecodeFormat};

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    NetClientBuilder::new()
        .headers(&[("User-Agent", create_user_agent_header())])?
        .build()?
        .into_registry(lua);
    TableBuilder::new(lua)?
        .with_function("jsonEncode", net_json_encode)?
        .with_function("jsonDecode", net_json_decode)?
        .with_async_function("request", net_request)?
        .with_async_function("socket", net_socket)?
        .with_async_function("serve", net_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}

fn net_json_encode<'lua>(
    lua: &'lua Lua,
    (val, pretty): (BString, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    EncodeDecodeConfig::from((EncodeDecodeFormat::Json, pretty.unwrap_or_default()))
        .serialize_to_string(lua, val)
}

fn net_json_decode<'lua>(lua: &'lua Lua, json: BString) -> LuaResult<LuaValue<'lua>> {
    EncodeDecodeConfig::from(EncodeDecodeFormat::Json).deserialize_from_string(lua, json)
}

async fn net_request(lua: &Lua, config: RequestConfig) -> LuaResult<LuaTable> {
    let client = NetClient::from_registry(lua);
    // NOTE: We spawn the request as a background task to free up resources in lua
    let res = lua.spawn(async move { client.request(config).await });
    res.await?.into_lua_table(lua)
}

async fn net_socket(lua: &Lua, url: String) -> LuaResult<LuaTable> {
    let (ws, _) = tokio_tungstenite::connect_async(url).await.into_lua_err()?;
    NetWebSocket::new(ws).into_lua_table(lua)
}

async fn net_serve<'lua>(
    lua: &'lua Lua,
    (port, config): (u16, ServeConfig<'lua>),
) -> LuaResult<LuaTable<'lua>> {
    serve(lua, port, config).await
}

fn net_url_encode<'lua>(
    lua: &'lua Lua,
    (lua_string, as_binary): (LuaString<'lua>, Option<bool>),
) -> LuaResult<LuaValue<'lua>> {
    if matches!(as_binary, Some(true)) {
        urlencoding::encode_binary(lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::encode(lua_string.to_str()?).into_lua(lua)
    }
}

fn net_url_decode<'lua>(
    lua: &'lua Lua,
    (lua_string, as_binary): (LuaString<'lua>, Option<bool>),
) -> LuaResult<LuaValue<'lua>> {
    if matches!(as_binary, Some(true)) {
        urlencoding::decode_binary(lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::decode(lua_string.to_str()?)
            .map_err(|e| LuaError::RuntimeError(format!("Encountered invalid encoding - {e}")))?
            .into_lua(lua)
    }
}
