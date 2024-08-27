use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<i32> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl CTypeConvert for CType<i32> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: i32 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<i32>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<i32>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<i32>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<i32> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<i32, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<i32, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<i32, f64>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "i32",
        CType::<i32>::new_with_libffi_type(lua, Type::i32(), Some("i32"))?,
    ))
}
