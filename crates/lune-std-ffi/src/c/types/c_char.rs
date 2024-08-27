use core::ffi::*;

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::{CType, CTypeCast, CTypeConvert};
use crate::ffi::ffi_platform::CHAR_IS_SIGNED;

impl CTypeConvert for CType<c_char> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: c_char = match value {
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
            *(ptr.cast::<c_char>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<c_char>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<c_char> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<c_char, c_float>(into_ctype, from, into)?
            .or(self.try_cast_num::<c_char, c_double>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_char, c_char>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_char, c_long>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_char, c_int>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_char, c_longlong>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn get_export(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "char",
        CType::<c_char>::new_with_libffi_type(
            lua,
            if CHAR_IS_SIGNED {
                Type::c_schar()
            } else {
                Type::c_uchar()
            },
            CHAR_IS_SIGNED,
            Some("char"),
        )?,
    ))
}
