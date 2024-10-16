#![allow(clippy::cargo_common_metadata)]

use bstr::BString;
use mlua::prelude::*;

use lune_utils::TableBuilder;

mod compress_decompress;
mod encode_decode;
mod hash;

pub use self::compress_decompress::{compress, decompress, CompressDecompressFormat};
pub use self::encode_decode::{decode, encode, EncodeDecodeConfig, EncodeDecodeFormat};
pub use self::hash::HashOptions;

/**
    Creates the `serde` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("encode", serde_encode)?
        .with_function("decode", serde_decode)?
        .with_async_function("compress", serde_compress)?
        .with_async_function("decompress", serde_decompress)?
        .with_function("hash", hash_message)?
        .with_function("hmac", hmac_message)?
        .build_readonly()
}

fn serde_encode<'lua>(
    lua: &'lua Lua,
    (format, value, pretty): (EncodeDecodeFormat, LuaValue<'lua>, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    let config = EncodeDecodeConfig::from((format, pretty.unwrap_or_default()));
    encode(value, lua, config)
}

fn serde_decode(lua: &Lua, (format, bs): (EncodeDecodeFormat, BString)) -> LuaResult<LuaValue> {
    let config = EncodeDecodeConfig::from(format);
    decode(bs, lua, config)
}

async fn serde_compress(
    lua: &Lua,
    (format, bs, level): (CompressDecompressFormat, BString, Option<i32>),
) -> LuaResult<LuaString> {
    let bytes = compress(bs, format, level).await?;
    lua.create_string(bytes)
}

async fn serde_decompress(
    lua: &Lua,
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
