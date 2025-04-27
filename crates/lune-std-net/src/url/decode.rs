use mlua::prelude::*;

pub fn decode(lua_string: LuaString, as_binary: bool) -> LuaResult<Vec<u8>> {
    if as_binary {
        Ok(urlencoding::decode_binary(&lua_string.as_bytes()).into_owned())
    } else {
        Ok(urlencoding::decode(&lua_string.to_str()?)
            .map_err(|e| LuaError::RuntimeError(format!("Encountered invalid encoding - {e}")))?
            .into_owned()
            .into_bytes())
    }
}
