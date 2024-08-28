use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<usize> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl CTypeConvert for CType<usize> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: usize = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<usize>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<usize>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<usize>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<usize> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<usize, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<usize, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<usize, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "usize",
        CType::<usize>::new_with_libffi_type(lua, Type::usize(), Some("usize"))?,
    ))
}
