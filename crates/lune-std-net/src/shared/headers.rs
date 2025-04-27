use std::collections::HashMap;

use hyper::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
    HeaderMap,
};

use lune_utils::TableBuilder;
use mlua::prelude::*;

pub fn create_user_agent_header(lua: &Lua) -> LuaResult<String> {
    let version_global = lua
        .globals()
        .get::<LuaString>("_VERSION")
        .expect("Missing _VERSION global");

    let version_global_str = version_global
        .to_str()
        .context("Invalid utf8 found in _VERSION global")?;

    let (package_name, full_version) = version_global_str.split_once(' ').unwrap();

    Ok(format!("{}/{}", package_name.to_lowercase(), full_version))
}

pub fn header_map_to_table(
    lua: &Lua,
    headers: HeaderMap,
    remove_content_headers: bool,
) -> LuaResult<LuaTable> {
    let mut string_map = HashMap::<String, Vec<String>>::new();

    for (name, value) in headers {
        if let Some(name) = name {
            if let Ok(value) = value.to_str() {
                string_map
                    .entry(name.to_string())
                    .or_default()
                    .push(value.to_owned());
            }
        }
    }

    hash_map_to_table(lua, string_map, remove_content_headers)
}

pub fn hash_map_to_table(
    lua: &Lua,
    map: impl IntoIterator<Item = (String, Vec<String>)>,
    remove_content_headers: bool,
) -> LuaResult<LuaTable> {
    let mut string_map = HashMap::<String, Vec<String>>::new();
    for (name, values) in map {
        let name = name.as_str();

        if remove_content_headers {
            let content_encoding_header_str = CONTENT_ENCODING.as_str();
            let content_length_header_str = CONTENT_LENGTH.as_str();
            if name == content_encoding_header_str || name == content_length_header_str {
                continue;
            }
        }

        for value in values {
            let value = value.as_str();
            string_map
                .entry(name.to_owned())
                .or_default()
                .push(value.to_owned());
        }
    }

    let mut builder = TableBuilder::new(lua.clone())?;
    for (name, mut values) in string_map {
        if values.len() == 1 {
            let value = values.pop().unwrap().into_lua(lua)?;
            builder = builder.with_value(name, value)?;
        } else {
            let values = TableBuilder::new(lua.clone())?
                .with_sequential_values(values)?
                .build_readonly()?
                .into_lua(lua)?;
            builder = builder.with_value(name, values)?;
        }
    }

    builder.build_readonly()
}

pub fn table_to_hash_map(
    tab: LuaTable,
    tab_origin_key: &'static str,
) -> LuaResult<HashMap<String, Vec<String>>> {
    let mut map = HashMap::new();

    for pair in tab.pairs::<String, LuaValue>() {
        let (key, value) = pair?;
        match value {
            LuaValue::String(s) => {
                map.insert(key, vec![s.to_str()?.to_owned()]);
            }
            LuaValue::Table(t) => {
                let mut values = Vec::new();
                for value in t.sequence_values::<LuaString>() {
                    values.push(value?.to_str()?.to_owned());
                }
                map.insert(key, values);
            }
            _ => {
                return Err(LuaError::runtime(format!(
                    "Value for '{tab_origin_key}' must be a string or array of strings",
                )))
            }
        }
    }

    Ok(map)
}
