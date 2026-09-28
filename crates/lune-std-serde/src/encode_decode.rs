use mlua::prelude::*;

use saphyr::{LoadableYamlNode, MappingOwned, ScalarOwned, Yaml, YamlEmitter, YamlOwned};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
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

#[derive(Debug)]
struct YamlValue(YamlOwned);

struct YamlValueVisitor;

impl<'de> Visitor<'de> for YamlValueVisitor {
    type Value = YamlOwned;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a value that can be represented as YAML")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::Boolean(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::Integer(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let value = i64::try_from(value).map_err(E::custom)?;
        Ok(YamlOwned::Value(ScalarOwned::Integer(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::FloatingPoint(value.into())))
    }

    fn visit_char<E>(self, value: char) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::String(value.into())))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::String(value.into())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(YamlOwned::Value(ScalarOwned::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = Vec::with_capacity(sequence.size_hint().unwrap_or_default());
        while let Some(value) = sequence.next_element::<YamlValue>()? {
            result.push(value.0);
        }
        Ok(YamlOwned::Sequence(result))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut result = MappingOwned::new();
        while let Some((key, value)) = map.next_entry::<YamlValue, YamlValue>()? {
            result.insert(key.0, value.0);
        }
        Ok(YamlOwned::Mapping(result))
    }
}

impl<'de> Deserialize<'de> for YamlValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(YamlValueVisitor).map(Self)
    }
}

fn yaml_to_lua(lua: &Lua, value: &YamlOwned) -> LuaResult<LuaValue> {
    match value {
        YamlOwned::Representation(value, _, _) => lua.create_string(value).map(LuaValue::String),
        YamlOwned::Value(ScalarOwned::Null) => Ok(LuaValue::Nil),
        YamlOwned::Value(ScalarOwned::Boolean(value)) => Ok(LuaValue::Boolean(*value)),
        YamlOwned::Value(ScalarOwned::Integer(value)) => Ok(LuaValue::Integer(*value)),
        YamlOwned::Value(ScalarOwned::FloatingPoint(value)) => {
            Ok(LuaValue::Number(value.into_inner()))
        }
        YamlOwned::Value(ScalarOwned::String(value)) => {
            lua.create_string(value).map(LuaValue::String)
        }
        YamlOwned::Sequence(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (index, value) in values.iter().enumerate() {
                table.raw_set(index + 1, yaml_to_lua(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        YamlOwned::Mapping(values) => {
            let table = lua.create_table_with_capacity(0, values.len())?;
            for (key, value) in values {
                let key = yaml_to_lua(lua, key)?;
                if matches!(key, LuaValue::Nil)
                    || matches!(key, LuaValue::Number(value) if value.is_nan())
                {
                    return Err(LuaError::RuntimeError(
                        "YAML mappings cannot contain null or NaN keys".to_string(),
                    ));
                }
                table.raw_set(key, yaml_to_lua(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        YamlOwned::Tagged(_, value) => yaml_to_lua(lua, value),
        YamlOwned::Alias(_) => Err(LuaError::RuntimeError(
            "YAML aliases are not supported".to_string(),
        )),
        YamlOwned::BadValue => Err(LuaError::RuntimeError(
            "YAML contains an invalid value".to_string(),
        )),
    }
}

/**
    An encoding and decoding format supported by Lune.

    Encode / decode in this case is synonymous with serialize / deserialize.
*/
#[derive(Debug, Clone, Copy)]
pub enum EncodeDecodeFormat {
    Json,
    JsonC,
    Yaml,
    Toml,
}

impl FromLua for EncodeDecodeFormat {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = &value {
            match s.to_string_lossy().to_ascii_lowercase().trim() {
                "json" => Ok(Self::Json),
                "jsonc" => Ok(Self::JsonC),
                "yaml" => Ok(Self::Yaml),
                "toml" => Ok(Self::Toml),
                kind => Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "EncodeDecodeFormat".to_string(),
                    message: Some(format!(
                        "Invalid format '{kind}', valid formats are:  json, yaml, toml"
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EncodeDecodeFormat".to_string(),
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
pub fn encode(value: LuaValue, lua: &Lua, config: EncodeDecodeConfig) -> LuaResult<LuaString> {
    let bytes = match config.format {
        EncodeDecodeFormat::Json | EncodeDecodeFormat::JsonC => {
            let serialized: JsonValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
            if config.pretty {
                serde_json::to_vec_pretty(&serialized).into_lua_err()?
            } else {
                serde_json::to_vec(&serialized).into_lua_err()?
            }
        }
        EncodeDecodeFormat::Yaml => {
            let serialized: YamlValue = lua.from_value_with(value, LUA_DESERIALIZE_OPTIONS)?;
            let value = Yaml::from(&serialized.0);
            let mut output = String::new();
            YamlEmitter::new(&mut output).dump(&value).into_lua_err()?;
            output
                .strip_prefix("---\n")
                .unwrap_or(&output)
                .as_bytes()
                .to_vec()
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
        EncodeDecodeFormat::JsonC => {
            let string: String = String::from_utf8(bytes.to_vec()).into_lua_err()?;
            let value: JsonValue =
                jsonc_parser::parse_to_serde_value(&string, &jsonc_parser::ParseOptions::default())
                    .into_lua_err()?;
            lua.to_value_with(&value, LUA_SERIALIZE_OPTIONS)
        }
        EncodeDecodeFormat::Yaml => {
            let string: String = String::from_utf8(bytes.to_vec()).into_lua_err()?;
            let mut documents = YamlOwned::load_from_str(&string).into_lua_err()?;
            if documents.len() != 1 {
                return Err(LuaError::RuntimeError(format!(
                    "expected exactly one YAML document, got {}",
                    documents.len()
                )));
            }
            yaml_to_lua(lua, &documents.remove(0))
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
