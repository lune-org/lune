use mlua::prelude::*;

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

// NOTE: These are options for going from other format -> lua ("serializing" lua values)
const LUA_SERIALIZE_OPTIONS: LuaSerializeOptions = LuaSerializeOptions::new()
    .set_array_metatable(false)
    .serialize_none_to_null(false)
    .serialize_unit_to_null(false);

// NOTE: These are options for going from lua -> other format ("deserializing" lua values)
const LUA_DESERIALIZE_OPTIONS: LuaDeserializeOptions = LuaDeserializeOptions::new()
    .sort_keys(true)
    .deny_recursive_tables(false)
    .deny_unsupported_types(true);

/**
    An encoding and decoding format supported by Lune.

    Encode / decode in this case is synonymous with serialize / deserialize.
*/
#[derive(Debug, Clone, Copy)]
pub enum EncodeDecodeFormat {
    Json,
    Yaml,
    Toml,
}

impl<'lua> FromLua<'lua> for EncodeDecodeFormat {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = &value {
            match s.to_string_lossy().to_ascii_lowercase().trim() {
                "json" => Ok(Self::Json),
                "yaml" => Ok(Self::Yaml),
                "toml" => Ok(Self::Toml),
                kind => Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "EncodeDecodeFormat",
                    message: Some(format!(
                        "Invalid format '{kind}', valid formats are:  json, yaml, toml"
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EncodeDecodeFormat",
                message: None,
            })
        }
    }
}

/**
    Configuration for encoding and decoding values.

    Encoding / decoding in this case is synonymous with serialize / deserialize.
*/
#[derive(Debug, Clone, Copy)]
pub struct EncodeDecodeConfig {
    pub format: EncodeDecodeFormat,
    pub pretty: bool,
}

impl From<EncodeDecodeFormat> for EncodeDecodeConfig {
    fn from(format: EncodeDecodeFormat) -> Self {
        Self {
            format,
            pretty: false,
        }
    }
}

impl From<(EncodeDecodeFormat, bool)> for EncodeDecodeConfig {
    fn from(value: (EncodeDecodeFormat, bool)) -> Self {
        Self {
            format: value.0,
            pretty: value.1,
        }
    }
}

/**
    Encodes / serializes the given value into a string, using the specified configuration.

    # Errors

    Errors when the encoding fails.
*/
pub fn encode<'lua>(
    value: LuaValue<'lua>,
    lua: &'lua Lua,
    config: EncodeDecodeConfig,
) -> LuaResult<LuaString<'lua>> {
    let bytes = match config.format {
        EncodeDecodeFormat::Json => {
            let serialized: JsonValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
            if config.pretty {
                serde_json::to_vec_pretty(&serialized).into_lua_err()?
            } else {
                serde_json::to_vec(&serialized).into_lua_err()?
            }
        }
        EncodeDecodeFormat::Yaml => {
            let serialized: YamlValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
            let mut writer = Vec::with_capacity(128);
            serde_yaml::to_writer(&mut writer, &serialized).into_lua_err()?;
            writer
        }
        EncodeDecodeFormat::Toml => {
            let serialized: TomlValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
            let s = if config.pretty {
                toml::to_string_pretty(&serialized).into_lua_err()?
            } else {
                toml::to_string(&serialized).into_lua_err()?
            };
            s.as_bytes().to_vec()
        }
    };
    lua.create_string(bytes)
}

/**
    Decodes / deserializes the given string into a value, using the specified configuration.

    # Errors

    Errors when the decoding fails.
*/
pub fn decode(
    bytes: impl AsRef<[u8]>,
    lua: &Lua,
    config: EncodeDecodeConfig,
) -> LuaResult<LuaValue> {
    let bytes = bytes.as_ref();
    match config.format {
        EncodeDecodeFormat::Json => {
            let value: JsonValue = serde_json::from_slice(bytes).into_lua_err()?;
            lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
        }
        EncodeDecodeFormat::Yaml => {
            let value: YamlValue = serde_yaml::from_slice(bytes).into_lua_err()?;
            lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
        }
        EncodeDecodeFormat::Toml => {
            if let Ok(s) = String::from_utf8(bytes.to_vec()) {
                let value: TomlValue = toml::from_str(&s).into_lua_err()?;
                lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
            } else {
                Err(LuaError::RuntimeError(
                    "TOML must be valid utf-8".to_string(),
                ))
            }
        }
    }
}
