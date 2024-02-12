#![allow(unused_variables)]

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

use super::serde::encode_decode::{EncodeDecodeConfig, EncodeDecodeFormat};

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
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

fn _create_user_agent_header() -> String {
    let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
        .trim_start_matches("https://github.com/")
        .split_once('/')
        .unwrap();
    format!("{github_owner}-{github_repo}-cli")
}

fn net_json_encode<'lua>(
    lua: &'lua Lua,
    (val, pretty): (LuaValue<'lua>, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    EncodeDecodeConfig::from((EncodeDecodeFormat::Json, pretty.unwrap_or_default()))
        .serialize_to_string(lua, val)
}

fn net_json_decode<'lua>(lua: &'lua Lua, json: LuaString<'lua>) -> LuaResult<LuaValue<'lua>> {
    EncodeDecodeConfig::from(EncodeDecodeFormat::Json).deserialize_from_string(lua, json)
}

async fn net_request<'lua>(lua: &'lua Lua, config: ()) -> LuaResult<LuaTable<'lua>> {
    unimplemented!()
}

async fn net_socket<'lua>(lua: &'lua Lua, url: String) -> LuaResult<LuaTable> {
    unimplemented!()
}

async fn net_serve<'lua>(lua: &'lua Lua, (port, config): (u16, ())) -> LuaResult<LuaTable<'lua>> {
    unimplemented!()
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
