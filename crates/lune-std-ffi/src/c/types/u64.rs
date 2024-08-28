use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<u64> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl CTypeConvert for CType<u64> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: u64 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<u64>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u64>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<u64>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<u64> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<u64, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<u64, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u64, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "u64",
        CType::<u64>::new_with_libffi_type(lua, Type::u64(), Some("u64"))?,
    ))
}
