use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::*;

impl CTypeSignedness for CType<u8> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl CTypeConvert for CType<u8> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value: u8 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::String(t) => t.as_bytes().first().map_or(0, u8::to_owned).as_(),
            _ => {
                return Err(LuaError::external(format!(
                    "Argument LuaValue expected a Integer or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(ptr.cast::<u8>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<u8>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CTypeCast for CType<u8> {
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<u8, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<u8, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<u8, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "u8",
        CType::<u8>::new_with_libffi_type(lua, Type::u8(), Some("u8"))?,
    ))
}
