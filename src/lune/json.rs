use mlua::{Error, Lua, LuaSerdeExt, Result, UserData, UserDataMethods, Value};

pub struct LuneJson();

impl LuneJson {
    pub fn new() -> Self {
        Self()
    }
}

impl UserData for LuneJson {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("encode", json_encode);
        methods.add_function("decode", json_decode);
    }
}

fn json_encode(_: &Lua, (val, pretty): (Value, Option<bool>)) -> Result<String> {
    if let Some(true) = pretty {
        Ok(serde_json::to_string_pretty(&val).map_err(Error::external)?)
    } else {
        Ok(serde_json::to_string(&val).map_err(Error::external)?)
    }
}

fn json_decode(lua: &Lua, json: String) -> Result<Value> {
    Ok(lua.to_value(&json)?)
}
