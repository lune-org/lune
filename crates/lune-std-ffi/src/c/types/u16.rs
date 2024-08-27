use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<u16> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl CTypeConvert for CType<u16> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: u16 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<u16>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u16>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<u16>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CType<u16> {}

impl CTypeCast for CType<u16> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<u16, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<u16, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u16, f64>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}
