use std::collections::HashMap;

use hyper::header::{CONTENT_ENCODING, CONTENT_LENGTH};
use reqwest::header::HeaderMap;

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

pub fn create_user_agent_header() -> String {
    let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
        .trim_start_matches("https://github.com/")
        .split_once('/')
        .unwrap();
    format!("{github_owner}-{github_repo}-cli")
}

pub fn header_map_to_table(
    lua: &Lua,
    headers: HeaderMap,
    remove_content_headers: bool,
) -> LuaResult<LuaTable> {
    let mut res_headers: HashMap<String, Vec<String>> = HashMap::new();
    for (name, value) in headers.iter() {
        let name = name.as_str();
        let value = value.to_str().unwrap().to_owned();
        if let Some(existing) = res_headers.get_mut(name) {
            existing.push(value);
        } else {
            res_headers.insert(name.to_owned(), vec![value]);
        }
    }

    if remove_content_headers {
        let content_encoding_header_str = CONTENT_ENCODING.as_str();
        let content_length_header_str = CONTENT_LENGTH.as_str();
        res_headers.retain(|name, _| {
            name != content_encoding_header_str && name != content_length_header_str
        });
    }

    let mut builder = TableBuilder::new(lua)?;
    for (name, mut values) in res_headers {
        if values.len() == 1 {
            let value = values.pop().unwrap().into_lua(lua)?;
            builder = builder.with_value(name, value)?;
        } else {
            let values = TableBuilder::new(lua)?
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
