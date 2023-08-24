use mlua::prelude::*;

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

const LUA_SERIALIZE_OPTIONS: LuaSerializeOptions = LuaSerializeOptions::new()
    .set_array_metatable(false)
    .serialize_none_to_null(false)
    .serialize_unit_to_null(false);

const LUA_DESERIALIZE_OPTIONS: LuaDeserializeOptions = LuaDeserializeOptions::new()
    .sort_keys(true)
    .deny_recursive_tables(false)
    .deny_unsupported_types(true);

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

#[derive(Debug, Clone, Copy)]
pub struct EncodeDecodeConfig {
    pub format: EncodeDecodeFormat,
    pub pretty: bool,
}

impl EncodeDecodeConfig {
    pub fn serialize_to_string<'lua>(
        self,
        lua: &'lua Lua,
        value: LuaValue<'lua>,
    ) -> LuaResult<LuaString<'lua>> {
        let bytes = match self.format {
            EncodeDecodeFormat::Json => {
                let serialized: JsonValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
                if self.pretty {
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
                let s = if self.pretty {
                    toml::to_string_pretty(&serialized).into_lua_err()?
                } else {
                    toml::to_string(&serialized).into_lua_err()?
                };
                s.as_bytes().to_vec()
            }
        };
        lua.create_string(bytes)
    }

    pub fn deserialize_from_string<'lua>(
        self,
        lua: &'lua Lua,
        string: LuaString<'lua>,
    ) -> LuaResult<LuaValue<'lua>> {
        let bytes = string.as_bytes();
        match self.format {
            EncodeDecodeFormat::Json => {
                let value: JsonValue = serde_json::from_slice(bytes).into_lua_err()?;
                lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
            }
            EncodeDecodeFormat::Yaml => {
                let value: YamlValue = serde_yaml::from_slice(bytes).into_lua_err()?;
                lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
            }
            EncodeDecodeFormat::Toml => {
                if let Ok(s) = string.to_str() {
                    let value: TomlValue = toml::from_str(s).into_lua_err()?;
                    lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
                } else {
                    Err(LuaError::RuntimeError(
                        "TOML must be valid utf-8".to_string(),
                    ))
                }
            }
        }
    }
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
