use core::ffi::*;

use libffi::middle::Type;
use mlua::prelude::*;

use super::super::c_type::{CType, CTypeCast, CTypeConvert};

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

impl CType<c_int> {}

impl CTypeCast for CType<c_int> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<c_int, c_float>(into_ctype, from, into)?
            .or(self.try_cast_num::<c_int, c_double>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_int, c_char>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_int, c_long>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_int, c_int>(into_ctype, from, into)?)
            .or(self.try_cast_num::<c_int, c_longlong>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

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
