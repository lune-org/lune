#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

pub(crate) mod body;
pub(crate) mod client;
pub(crate) mod server;
pub(crate) mod shared;
pub(crate) mod url;

use crate::shared::tcp::Tcp;

use self::{
    client::{stream::WsStream, tcp::TcpConfig},
    server::config::ServeConfig,
    shared::{request::Request, response::Response, websocket::Websocket},
};

pub use self::client::fetch;

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
    let submodule_http = TableBuilder::new(lua.clone())?
        .with_async_function("request", net_http_request)?
        .with_async_function("socket", net_http_socket)?
        .with_async_function("serve", net_http_serve)?
        .build_readonly()?;

    let submodule_tcp = TableBuilder::new(lua.clone())?
        .with_async_function("connect", net_tcp_connect)?
        .build_readonly()?;

    TableBuilder::new(lua)?
        .with_async_function("request", net_http_request)?
        .with_async_function("socket", net_http_socket)?
        .with_async_function("serve", net_http_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .with_value("http", submodule_http)?
        .with_value("tcp", submodule_tcp)?
        .build_readonly()
}

async fn net_http_request(lua: Lua, req: Request) -> LuaResult<Response> {
    self::client::send(req, lua).await
}

async fn net_http_socket(_: Lua, url: String) -> LuaResult<Websocket<WsStream>> {
    let url = url.parse().into_lua_err()?;
    self::client::connect_websocket(url).await
}

async fn net_http_serve(lua: Lua, (port, config): (u16, ServeConfig)) -> LuaResult<LuaTable> {
    self::server::serve(lua.clone(), port, config)
        .await?
        .into_lua_table(lua)
}

async fn net_tcp_connect(_: Lua, (host, port, config): (String, u16, TcpConfig)) -> LuaResult<Tcp> {
    self::client::connect_tcp(host, port, config).await
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
