use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    data::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<u16> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl FfiConvert for CTypeInfo<u16> {
    // Convert luavalue into data, then write into ptr
    unsafe fn value_into_data<'lua>(
        &self,
        _lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
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
            *(data_handle.get_pointer().byte_offset(offset).cast::<u16>()) = value;
        }
        Ok(())
    }
    unsafe fn value_from_data<'lua>(
        &self,
        lua: &'lua Lua,
        // _type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe {
            (*data_handle.get_pointer().byte_offset(offset).cast::<u16>()).into_lua(lua)?
        };
        Ok(value)
    }
}
