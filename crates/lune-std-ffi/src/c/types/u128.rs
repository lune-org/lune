use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<u128> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl CTypeConvert for CType<u128> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: u128 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<u128>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u128>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<u128>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<u128> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<u128, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<u128, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u128, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "u128",
        CType::<u128>::new_with_libffi_type(
            lua,
            Type::structure(vec![Type::u64(), Type::u64()]),
            Some("u128"),
        )?,
    ))
}
