use core::ffi::*;

use libffi::middle::Type;
use mlua::prelude::*;

use super::super::c_type::{CType, CTypeConvert, CTypeNumCast};
use crate::ffi::ffi_platform::CHAR_IS_SIGNED;

impl CTypeConvert for CType<c_char> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value = match value {
            LuaValue::Integer(t) => t,
            _ => {
                return Err(LuaError::external(format!(
                    "Integer expected, got {}",
                    value.type_name()
                )))
            }
        } as c_char;
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

impl CType<c_char> {
    fn cast(
        &self,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        Self::cast_userdata_if_type_match::<c_float>(into_ctype, from, into)?
            .or(Self::cast_userdata_if_type_match::<c_double>(
                into_ctype, from, into,
            )?)
            .or(Self::cast_userdata_if_type_match::<c_char>(
                into_ctype, from, into,
            )?)
            .or(Self::cast_userdata_if_type_match::<c_long>(
                into_ctype, from, into,
            )?)
            .ok_or_else(|| self.cast_failed_with(into_ctype))
    }
}

impl CTypeNumCast<c_char> for CType<c_char> {}

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
