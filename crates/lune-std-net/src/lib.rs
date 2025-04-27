#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

pub(crate) mod client;
pub(crate) mod server;
pub(crate) mod shared;
pub(crate) mod url;

#[allow(unused_imports)]
use self::{
    client::config::RequestConfig,
    server::config::ResponseConfig,
    shared::{request::Request, response::Response},
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
        // .with_async_function("socket", net_socket)?
        // .with_async_function("serve", net_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}

async fn net_request(lua: Lua, config: RequestConfig) -> LuaResult<Response> {
    self::client::send_request(Request::try_from(config)?, lua).await
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
