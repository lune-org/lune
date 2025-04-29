#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

pub(crate) mod client;
pub(crate) mod server;
pub(crate) mod shared;
pub(crate) mod url;

use self::{
    client::ws_stream::WsStream,
    server::config::ServeConfig,
    shared::{request::Request, response::Response, websocket::Websocket},
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
    TableBuilder::new(lua)?
        .with_async_function("request", net_request)?
        .with_async_function("socket", net_socket)?
        .with_async_function("serve", net_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}

async fn net_request(lua: Lua, req: Request) -> LuaResult<Response> {
    self::client::send_request(req, lua).await
}

async fn net_socket(_: Lua, url: String) -> LuaResult<Websocket<WsStream>> {
    let url = url.parse().into_lua_err()?;
    self::client::connect_websocket(url).await
}

async fn net_serve(lua: Lua, (port, config): (u16, ServeConfig)) -> LuaResult<LuaTable> {
    self::server::serve(lua.clone(), port, config)
        .await?
        .into_lua_table(lua)
}

fn net_url_encode(
    lua: &Lua,
    (lua_string, as_binary): (LuaString, Option<bool>),
) -> LuaResult<LuaString> {
    let as_binary = as_binary.unwrap_or_default();
    let bytes = self::url::encode(lua_string, as_binary)?;
    lua.create_string(bytes)
}

fn net_url_decode(
    lua: &Lua,
    (lua_string, as_binary): (LuaString, Option<bool>),
) -> LuaResult<LuaString> {
    let as_binary = as_binary.unwrap_or_default();
    let bytes = self::url::decode(lua_string, as_binary)?;
    lua.create_string(bytes)
}
