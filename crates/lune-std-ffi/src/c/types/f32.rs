use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<f32> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl CTypeConvert for CType<f32> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: f32 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<f32>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<f32>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<f32>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<f32> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<f32, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<f32, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<f32, f64>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}
