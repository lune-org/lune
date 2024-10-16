use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::c_type::CType;
use crate::ffi::{NativeConvert, NativeData, NativeSignedness};

impl NativeSignedness for CType<u8> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl NativeConvert for CType<u8> {
    // Convert luavalue into data, then write into ptr
    unsafe fn luavalue_into<'lua>(
        &self,
        _lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
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
            *(data_handle.get_pointer().byte_offset(offset).cast::<u8>()) = value;
        }
        Ok(())
    }

    // Read data from ptr, then convert into luavalue
    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value =
            unsafe { (*data_handle.get_pointer().byte_offset(offset).cast::<u8>()).into_lua(lua)? };
        Ok(value)
    }
}
