use mlua::prelude::*;

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

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
                if self.pretty {
                    serde_json::to_vec_pretty(&value).map_err(LuaError::external)?
                } else {
                    serde_json::to_vec(&value).map_err(LuaError::external)?
                }
            }
            EncodeDecodeFormat::Yaml => {
                let mut writer = Vec::with_capacity(128);
                serde_yaml::to_writer(&mut writer, &value).map_err(LuaError::external)?;
                writer
            }
            EncodeDecodeFormat::Toml => {
                let s = if self.pretty {
                    toml::to_string_pretty(&value).map_err(LuaError::external)?
                } else {
                    toml::to_string(&value).map_err(LuaError::external)?
                };
                s.as_bytes().to_vec()
            }
        };
        lua.create_string(&bytes)
    }

    pub fn deserialize_from_string<'lua>(
        self,
        lua: &'lua Lua,
        string: LuaString<'lua>,
    ) -> LuaResult<LuaValue<'lua>> {
        let bytes = string.as_bytes();
        match self.format {
            EncodeDecodeFormat::Json => {
                let value: JsonValue = serde_json::from_slice(bytes).map_err(LuaError::external)?;
                lua.to_value(&value)
            }
            EncodeDecodeFormat::Yaml => {
                let value: YamlValue = serde_yaml::from_slice(bytes).map_err(LuaError::external)?;
                lua.to_value(&value)
            }
            EncodeDecodeFormat::Toml => {
                if let Ok(s) = string.to_str() {
                    let value: TomlValue = toml::from_str(s).map_err(LuaError::external)?;
                    lua.to_value(&value)
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
