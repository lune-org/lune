use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use crate::{
    c::type_info::CTypeInfo,
    ffi::{FfiConvert, FfiData, FfiSignedness},
};

impl FfiSignedness for CTypeInfo<i8> {
    fn get_signedness(&self) -> bool {
        true
    }
}

impl FfiConvert for CTypeInfo<i8> {
    unsafe fn value_into_data<'lua>(
        &self,
        _lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let value: i8 = match value {
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
            *(data_handle.get_pointer().byte_offset(offset).cast::<i8>()) = value;
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
        let value =
            unsafe { (*data_handle.get_pointer().byte_offset(offset).cast::<i8>()).into_lua(lua)? };
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
        *dst.get_pointer().byte_offset(dst_offset).cast::<i8>() =
            *src.get_pointer().byte_offset(src_offset).cast::<i8>();
        Ok(())
    }
    unsafe fn stringify_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<String> {
        Ok((*data_handle.get_pointer().byte_offset(offset).cast::<i8>()).to_string())
    }
}
