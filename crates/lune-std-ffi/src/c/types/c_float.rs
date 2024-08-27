use core::ffi::*;

use libffi::middle::Type;
use mlua::prelude::*;

use super::super::c_type::{CType, CTypeCast, CTypeConvert};
use num::ToPrimitive;

impl CTypeConvert for CType<c_float> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value = match value {
            LuaValue::Number(t) => t
                .to_f32()
                .ok_or(LuaError::external("number conversion failed"))?,
            LuaValue::Integer(t) => t
                .to_f32()
                .ok_or(LuaError::external("number conversion failed"))?,
            _ => {
                return Err(LuaError::external(format!(
                    "Integer expected, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<c_float>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<c_float>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<c_float> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<c_float, c_float>(into_ctype, from, into)?
            .or(self.try_cast_num::<c_float, c_double>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_float, c_char>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_float, c_long>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_float, c_int>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_float, c_longlong>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn get_export(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "float",
        CType::<c_float>::new_with_libffi_type(lua, Type::f32(), true, Some("float"))?,
    ))
}
