use core::ffi::*;

use libffi::middle::Type;
use mlua::prelude::*;

use super::super::c_type::{CType, CTypeConvert, CTypeNumCast};

impl CTypeConvert for CType<c_int> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value = match value {
            LuaValue::Integer(t) => t,
            _ => {
                return Err(LuaError::external(format!(
                    "Integer expected, got {}",
                    value.type_name()
                )))
            }
        } as c_int;
        unsafe {
            *(ptr.cast::<c_int>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<c_int>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CType<c_int> {
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

impl CTypeNumCast<c_int> for CType<c_int> {}

pub fn get_export(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "int",
        CType::<c_int>::new_with_libffi_type(
            lua,
            Type::c_int(),
            c_int::MIN.unsigned_abs() != 0,
            Some("int"),
        )?,
    ))
}
