use std::cell::Ref;

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::CType;
use crate::ffi::{NativeConvert, NativeDataHandle, NativeSignedness};

impl NativeSignedness for CType<f32> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl NativeConvert for CType<f32> {
    unsafe fn luavalue_into<'lua>(
        &self,
        _lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeDataHandle>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
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
            *(data_handle.get_pointer(offset).cast::<f32>()) = value;
        }
        Ok(())
    }
    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeDataHandle>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe { (*data_handle.get_pointer(offset).cast::<f32>()).into_lua(lua)? };
        Ok(value)
    }
}

pub fn create_type(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok((
        "f32",
        CType::<f32>::new_with_libffi_type(lua, Type::f32(), Some("f32"))?,
    ))
}
