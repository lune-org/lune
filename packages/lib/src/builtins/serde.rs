use mlua::prelude::*;

use crate::lua::{
    serde::{
        compress, decompress, CompressDecompressFormat, EncodeDecodeConfig, EncodeDecodeFormat,
    },
    table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("encode", serde_encode)?
        .with_function("decode", serde_decode)?
        .with_async_function("compress", serde_compress)?
        .with_async_function("decompress", serde_decompress)?
        .build_readonly()
}

fn serde_encode<'a>(
    lua: &'static Lua,
    (format, val, pretty): (EncodeDecodeFormat, LuaValue<'a>, Option<bool>),
) -> LuaResult<LuaString<'a>> {
    let config = EncodeDecodeConfig::from((format, pretty.unwrap_or_default()));
    config.serialize_to_string(lua, val)
}

fn serde_decode<'a>(
    lua: &'static Lua,
    (format, str): (EncodeDecodeFormat, LuaString<'a>),
) -> LuaResult<LuaValue<'a>> {
    let config = EncodeDecodeConfig::from(format);
    config.deserialize_from_string(lua, str)
}

async fn serde_compress<'a>(
    lua: &'static Lua,
    (format, str): (CompressDecompressFormat, LuaString<'a>),
) -> LuaResult<LuaString<'a>> {
    let bytes = compress(format, str).await?;
    lua.create_string(bytes)
}

async fn serde_decompress<'a>(
    lua: &'static Lua,
    (format, str): (CompressDecompressFormat, LuaString<'a>),
) -> LuaResult<LuaString<'a>> {
    let bytes = decompress(format, str).await?;
    lua.create_string(bytes)
}
