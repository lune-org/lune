use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    ffi::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<isize> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl FfiConvert for CTypeInfo<isize> {
    unsafe fn value_into_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
        value: LuaValue,
    ) -> LuaResult<()> {
        let value: isize = match value {
            LuaValue::Integer(t) => t.as_(),
            LuaValue::Number(t) => t.as_(),
            LuaValue::String(t) => t
                .to_string_lossy()
                .parse::<isize>()
                .map_err(LuaError::external)?,
            _ => {
                return Err(LuaError::external(format!(
                    "Value must be a Integer, Number or String, got {}",
                    value.type_name()
                )))
            }
        };
        unsafe {
            data_handle
                .get_inner_pointer()
                .byte_offset(offset)
                .cast::<isize>()
                .write_unaligned(value);
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
            data_handle
                .get_inner_pointer()
                .byte_offset(offset)
                .cast::<isize>()
                .read_unaligned()
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
        dst.get_inner_pointer()
            .byte_offset(dst_offset)
            .cast::<isize>()
            .write_unaligned(
                src.get_inner_pointer()
                    .byte_offset(src_offset)
                    .cast::<isize>()
                    .read_unaligned(),
            );
        Ok(())
    }
    unsafe fn stringify_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
    ) -> LuaResult<String> {
        Ok(data_handle
            .get_inner_pointer()
            .byte_offset(offset)
            .cast::<isize>()
            .read_unaligned()
            .to_string())
    }
}
