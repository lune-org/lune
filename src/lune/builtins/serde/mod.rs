use mlua::prelude::*;

pub(super) mod compress_decompress;
pub(super) mod crypto;
pub(super) mod encode_decode;

use compress_decompress::{compress, decompress, CompressDecompressFormat};
use crypto::Crypto;
use encode_decode::{EncodeDecodeConfig, EncodeDecodeFormat};

use crate::lune::util::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("encode", serde_encode)?
        .with_function("decode", serde_decode)?
        .with_async_function("compress", serde_compress)?
        .with_async_function("decompress", serde_decompress)?
        .with_value(
            "crypto",
            TableBuilder::new(lua)?
                .with_function("sha1", |_, content: Option<String>| {
                    Ok(Crypto::sha1(content))
                })?
                .with_function("sha256", |_, content: Option<String>| {
                    Ok(Crypto::sha256(content))
                })?
                .with_function("sha512", |_, content: Option<String>| {
                    Ok(Crypto::sha512(content))
                })?
                .with_function("md5", |_, content: Option<String>| Ok(Crypto::md5(content)))?
                .build()?,
        )?
        .build_readonly()
}

fn serde_encode<'lua>(
    lua: &'lua Lua,
    (format, val, pretty): (EncodeDecodeFormat, LuaValue<'lua>, Option<bool>),
) -> LuaResult<LuaString<'lua>> {
    let config = EncodeDecodeConfig::from((format, pretty.unwrap_or_default()));
    config.serialize_to_string(lua, val)
}

fn serde_decode<'lua>(
    lua: &'lua Lua,
    (format, str): (EncodeDecodeFormat, LuaString<'lua>),
) -> LuaResult<LuaValue<'lua>> {
    let config = EncodeDecodeConfig::from(format);
    config.deserialize_from_string(lua, str)
}

async fn serde_compress<'lua>(
    lua: &'lua Lua,
    (format, str): (CompressDecompressFormat, LuaString<'lua>),
) -> LuaResult<LuaString<'lua>> {
    let bytes = compress(format, str).await?;
    lua.create_string(bytes)
}

async fn serde_decompress<'lua>(
    lua: &'lua Lua,
    (format, str): (CompressDecompressFormat, LuaString<'lua>),
) -> LuaResult<LuaString<'lua>> {
    let bytes = decompress(format, str).await?;
    lua.create_string(bytes)
}
