#![allow(clippy::cargo_common_metadata)]

use bstr::BString;
use mlua::prelude::*;

use lune_utils::TableBuilder;

mod compress_decompress;
mod encode_decode;
mod hash;

pub use self::compress_decompress::{CompressDecompressFormat, compress, decompress};
pub use self::encode_decode::{EncodeDecodeConfig, EncodeDecodeFormat, decode, encode};
pub use self::hash::HashOptions;

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `serde` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `serde` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("encode", serde_encode)?
        .with_function("decode", serde_decode)?
        .with_async_function("compress", serde_compress)?
        .with_async_function("decompress", serde_decompress)?
        .with_function("hash", hash_message)?
        .with_function("hmac", hmac_message)?
        .build_readonly()
}

fn serde_encode(
    lua: &Lua,
    (format, value, pretty): (EncodeDecodeFormat, LuaValue, Option<bool>),
) -> LuaResult<LuaString> {
    let config = EncodeDecodeConfig::from((format, pretty.unwrap_or_default()));
    encode(value, lua, config)
}

fn serde_decode(lua: &Lua, (format, bs): (EncodeDecodeFormat, BString)) -> LuaResult<LuaValue> {
    let config = EncodeDecodeConfig::from(format);
    decode(bs, lua, config)
}

async fn serde_compress(
    lua: Lua,
    (format, bs, level): (CompressDecompressFormat, BString, Option<i32>),
) -> LuaResult<LuaString> {
    let bytes = compress(bs, format, level).await?;
    lua.create_string(bytes)
}

async fn serde_decompress(
    lua: Lua,
    (format, bs): (CompressDecompressFormat, BString),
) -> LuaResult<LuaString> {
    let bytes = decompress(bs, format).await?;
    lua.create_string(bytes)
}

fn hash_message(lua: &Lua, options: HashOptions) -> LuaResult<LuaString> {
    lua.create_string(options.hash())
}

fn hmac_message(lua: &Lua, options: HashOptions) -> LuaResult<LuaString> {
    lua.create_string(options.hmac()?)
}
