use mlua::{IntoLua, Lua, Result, Value};

const BYTES_TO_BUF_IMPL: &str = r#"
    local tbl = select(1, ...)
    local buf = buffer.create(#tbl * 4) -- Each u32 is 4 bytes

    for offset, byte in tbl do
        buffer.writeu32(buf, offset, byte)
    end

    return buf
"#;

const BUF_TO_STR_IMPL: &str = "return buffer.tostring(select(1, ...))";

pub fn create_lua_buffer(lua: &Lua, bytes: impl AsRef<[u8]>) -> Result<Value> {
    let lua_bytes = bytes.as_ref().into_lua(lua)?;

    let buf_constructor = lua.load(BYTES_TO_BUF_IMPL).into_function()?;

    buf_constructor.call::<_, Value>(lua_bytes)
}

pub fn buf_to_str(lua: &Lua, buf: Value<'_>) -> Result<String> {
    let str_constructor = lua.load(BUF_TO_STR_IMPL).into_function()?;

    str_constructor.call(buf)
}
