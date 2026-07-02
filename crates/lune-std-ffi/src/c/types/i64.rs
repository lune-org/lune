
use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    ffi::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<i64> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl FfiConvert for CTypeInfo<i64> {
    unsafe fn value_into_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
        value: LuaValue,
    ) -> LuaResult<()> {
        let value: i64 = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<i64>()
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
                .cast::<i64>()) = value;
        }
        Ok(())
    }
    unsafe fn value_from_data(
        &self,
        lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
    ) -> LuaResult<LuaValue> {
        let value = unsafe {
            (*data_handle
                .get_inner_pointer()
                .byte_offset(offset)
                .cast::<i64>())
            .into_lua(lua)?
        };
        Ok(value)
    }
    unsafe fn copy_data(
        &self,
        _lua: &Lua,
        dst_offset: isize,
        src_offset: isize,
        dst: &dyn FfiData,
        src: &dyn FfiData,
    ) -> LuaResult<()> {
        *dst.get_inner_pointer()
            .byte_offset(dst_offset)
            .cast::<i64>() = *src
            .get_inner_pointer()
            .byte_offset(src_offset)
            .cast::<i64>();
        Ok(())
    }
    unsafe fn stringify_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
    ) -> LuaResult<String> {
        Ok((*data_handle
            .get_inner_pointer()
            .byte_offset(offset)
            .cast::<i64>())
        .to_string())
    }
}
