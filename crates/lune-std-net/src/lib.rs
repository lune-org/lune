#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

mod request;
mod serve;
mod socket;
mod url;

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
        // .with_async_function("request", net_request)?
        // .with_async_function("socket", net_socket)?
        // .with_async_function("serve", net_serve)?
        // .with_function("urlEncode", net_url_encode)?
        // .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}
