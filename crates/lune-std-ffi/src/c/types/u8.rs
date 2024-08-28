use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::{CType, CTypeSignedness};
use crate::ffi::ffi_native::NativeConvert;

impl CTypeSignedness for CType<u8> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl NativeConvert for CType<u8> {
    // Convert luavalue into data, then write into ptr
    fn luavalue_into_ptr<'lua>(
        &self,
        _this: &LuaAnyUserData<'lua>,
        _lua: &'lua Lua,
        value: LuaValue<'lua>,
        ptr: *mut (),
    ) -> LuaResult<()> {
        let value: u8 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::String(t) => t.as_bytes().first().map_or(0, u8::to_owned).as_(),
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u8>()) = value;
        }
        Ok(())
    }

    // Read data from ptr, then convert into luavalue
    fn ptr_into_luavalue<'lua>(
        &self,
        _this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        ptr: *mut (),
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe { (*ptr.cast::<u8>()).into_lua(lua)? };
        Ok(value)
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "u8",
        CType::<u8>::new_with_libffi_type(lua, Type::u8(), Some("u8"))?,
    ))
}
