use mlua::prelude::*;

use crate::lua::{
    net::{EncodeDecodeConfig, EncodeDecodeFormat},
    table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("encode", serde_encode)?
        .with_function("decode", serde_decode)?
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
