use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap, Method,
};

use mlua::prelude::*;

pub fn lua_value_to_method(value: &LuaValue) -> LuaResult<Method> {
    match value {
        LuaValue::Nil => Ok(Method::GET),
        LuaValue::String(str) => {
            let bytes = str.as_bytes().trim_ascii().to_ascii_uppercase();
            Method::from_bytes(&bytes).into_lua_err()
        }
        LuaValue::Buffer(buf) => {
            let bytes = buf.to_vec().trim_ascii().to_ascii_uppercase();
            Method::from_bytes(&bytes).into_lua_err()
        }
        v => Err(LuaError::FromLuaConversionError {
            from: v.type_name(),
            to: "Method".to_string(),
            message: Some(format!(
                "Invalid method - expected string or buffer, got {}",
                v.type_name()
            )),
        }),
    }
}

pub fn lua_table_to_header_map(table: &LuaTable) -> LuaResult<HeaderMap> {
    let mut headers = HeaderMap::new();

    for pair in table.pairs::<LuaString, LuaString>() {
        let (key, val) = pair?;
        let key = HeaderName::from_bytes(&key.as_bytes()).into_lua_err()?;
        let val = HeaderValue::from_bytes(&val.as_bytes()).into_lua_err()?;
        headers.insert(key, val);
    }

    Ok(headers)
}
