use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    data::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<u8> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl FfiConvert for CTypeInfo<u8> {
    // Convert luavalue into data, then write into ptr
    unsafe fn value_into_data<'lua>(
        &self,
        _lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
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
    unsafe fn value_from_data<'lua>(
        &self,
        lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value =
            unsafe { (*data_handle.get_pointer().byte_offset(offset).cast::<u8>()).into_lua(lua)? };
        Ok(value)
    }
}
