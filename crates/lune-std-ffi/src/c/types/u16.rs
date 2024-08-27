use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::{CType, CTypeSignedness};
use crate::ffi::ffi_native::NativeConvert;

impl CTypeSignedness for CType<u16> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl NativeConvert for CType<u16> {
    // Convert luavalue into data, then write into ptr
    fn luavalue_into_ptr<'lua>(
        &self,
        _this: &LuaAnyUserData<'lua>,
        _lua: &'lua Lua,
        value: LuaValue<'lua>,
        ptr: *mut (),
    ) -> LuaResult<()> {
        let value: u16 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<u16>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u16>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue<'lua>(
        &self,
        _this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        ptr: *mut (),
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe { (*ptr.cast::<u16>()).into_lua(lua)? };
        Ok(value)
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "u16",
        CType::<u16>::new_with_libffi_type(lua, Type::u16(), Some("u16"))?,
    ))
}
