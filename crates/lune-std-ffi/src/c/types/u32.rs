use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    ffi::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<u32> {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl FfiConvert for CTypeInfo<u32> {
    unsafe fn value_into_data<'lua>(
        &self,
        _lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let value: u32 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<u32>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Value must be a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            *(data_handle
                .get_inner_pointer()
                .byte_offset(offset)
                .cast::<u32>()) = value;
        }
        Ok(())
    }
    unsafe fn value_from_data<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = unsafe {
            (*data_handle
                .get_inner_pointer()
                .byte_offset(offset)
                .cast::<u32>())
            .into_lua(lua)?
        };
        Ok(value)
    }
    unsafe fn copy_data(
        &self,
        _lua: &Lua,
        dst_offset: isize,
        src_offset: isize,
        dst: &Ref<dyn FfiData>,
        src: &Ref<dyn FfiData>,
    ) -> LuaResult<()> {
        *dst.get_inner_pointer()
            .byte_offset(dst_offset)
            .cast::<u32>() = *src
            .get_inner_pointer()
            .byte_offset(src_offset)
            .cast::<u32>();
        Ok(())
    }
    unsafe fn stringify_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<String> {
        Ok((*data_handle
            .get_inner_pointer()
            .byte_offset(offset)
            .cast::<f32>())
        .to_string())
    }
}
