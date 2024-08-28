use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<isize> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl CTypeConvert for CType<isize> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: isize = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<isize>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<isize>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<isize>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<isize> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<isize, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<isize, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<isize, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "isize",
        CType::<isize>::new_with_libffi_type(lua, Type::isize(), Some("isize"))?,
    ))
}
