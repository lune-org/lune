use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<i16> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl CTypeConvert for CType<i16> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: i16 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<i16>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<i16>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<i16>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<i16> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<i16, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<i16, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i16, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "i16",
        CType::<i16>::new_with_libffi_type(lua, libffi::middle::Type::i16(), Some("f32"))?,
    ))
}
