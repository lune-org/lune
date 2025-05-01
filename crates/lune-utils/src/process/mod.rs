use std::ffi::{OsStr, OsString};

use mlua::prelude::*;
use os_str_bytes::{OsStrBytes, OsStringBytes};

mod args;
mod env;
mod jit;

pub use self::args::ProcessArgs;
pub use self::env::ProcessEnv;
pub use self::jit::ProcessJitEnablement;

fn lua_value_to_os_string(res: LuaResult<LuaValue>, to: &'static str) -> LuaResult<OsString> {
    let (btype, bs) = match res {
        Ok(LuaValue::String(s)) => ("string", s.as_bytes().to_vec()),
        Ok(LuaValue::Buffer(b)) => ("buffer", b.to_vec()),
        res => {
            let vtype = match res {
                Ok(v) => v.type_name(),
                Err(_) => "unknown",
            };
            return Err(LuaError::FromLuaConversionError {
                from: vtype,
                to: String::from(to),
                message: Some(format!(
                    "Expected value to be a string or buffer, got '{vtype}'",
                )),
            });
        }
    };

    let Some(s) = OsString::from_io_vec(bs) else {
        return Err(LuaError::FromLuaConversionError {
            from: btype,
            to: String::from(to),
            message: Some(String::from("Expected {btype} to contain valid OS bytes")),
        });
    };

    Ok(s)
}

fn validate_os_key(key: &OsStr) -> LuaResult<()> {
    let Some(key) = key.to_io_bytes() else {
        return Err(LuaError::runtime("Key must be IO-safe"));
    };
    if key.is_empty() {
        Err(LuaError::runtime("Key must not be empty"))
    } else if key.contains(&b'=') {
        Err(LuaError::runtime(
            "Key must not contain the equals character '='",
        ))
    } else if key.contains(&b'\0') {
        Err(LuaError::runtime("Key must not contain the NUL character"))
    } else {
        Ok(())
    }
}

fn validate_os_value(val: &OsStr) -> LuaResult<()> {
    let Some(val) = val.to_io_bytes() else {
        return Err(LuaError::runtime("Value must be IO-safe"));
    };
    if val.contains(&b'\0') {
        Err(LuaError::runtime(
            "Value must not contain the NUL character",
        ))
    } else {
        Ok(())
    }
}

fn validate_os_pair((key, value): (&OsStr, &OsStr)) -> LuaResult<()> {
    validate_os_key(key)?;
    validate_os_value(value)?;
    Ok(())
}
