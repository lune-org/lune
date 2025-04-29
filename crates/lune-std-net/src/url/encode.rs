use mlua::prelude::*;

pub fn encode(lua_string: LuaString, as_binary: bool) -> LuaResult<Vec<u8>> {
    if as_binary {
        Ok(urlencoding::encode_binary(&lua_string.as_bytes())
            .into_owned()
            .into_bytes())
    } else {
        Ok(urlencoding::encode(&lua_string.to_str()?)
            .into_owned()
            .into_bytes())
    }
}
