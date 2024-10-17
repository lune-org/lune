use std::cell::Ref;

use libffi::middle::Type;
use mlua::prelude::*;

use super::{association_names::CPTR_INNER, ctype_helper, helper, method_provider};
use crate::{
    data::RefData,
    ffi::{
        association, libffi_helper::SIZE_OF_POINTER, FfiConvert, FfiData, FfiSignedness, FfiSize,
    },
};

pub struct CPtrInfo {
    inner_size: usize,
}

impl FfiSignedness for CPtrInfo {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl FfiSize for CPtrInfo {
    fn get_size(&self) -> usize {
        SIZE_OF_POINTER
    }
}
impl FfiConvert for CPtrInfo {
    // Convert luavalue into data, then write into ptr
    unsafe fn value_into_data<'lua>(
        &self,
        _lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        if let LuaValue::UserData(value_userdata) = value {
            if value_userdata.is::<RefData>() {
                let value_ref = value_userdata.borrow::<RefData>()?;
                value_ref
                    .check_boundary(0, self.inner_size)
                    .then_some(())
                    .ok_or_else(|| LuaError::external("boundary check failed"))?;
                *data_handle
                    .get_pointer()
                    .byte_offset(offset)
                    .cast::<*mut ()>() = value_ref.get_pointer();
                Ok(())
            } else {
                Err(LuaError::external("Ptr:into only allows FfiRef"))
            }
        } else {
            Err(LuaError::external("Conversion of pointer is not allowed"))
        }
    }

    // Read data from ptr, then convert into luavalue
    unsafe fn value_from_data<'lua>(
        &self,
        _lua: &'lua Lua,
        _offset: isize,
        _data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>> {
        Err(LuaError::external("Conversion of pointer is not allowed"))
    }
}

impl CPtrInfo {
    // Create pointer type with '.inner' field
    // inner can be CArr, CType or CStruct
    pub fn from_userdata<'lua>(
        lua: &'lua Lua,
        inner: &LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let value = lua.create_userdata(Self {
            inner_size: helper::get_size(inner)?,
        })?;

        association::set(lua, CPTR_INNER, &value, inner)?;

        Ok(value)
    }

    // Stringify CPtr with inner ctype
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::UserData(inner_userdata) = userdata.get("inner")? {
            let pretty_formatted = helper::pretty_format(lua, &inner_userdata)?;
            Ok(if ctype_helper::is_ctype(&inner_userdata) {
                pretty_formatted
            } else {
                format!(" {pretty_formatted} ")
            })
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }

    // Return void*
    pub fn get_type() -> Type {
        Type::pointer()
    }
}

impl LuaUserData for CPtrInfo {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, _| Ok(size_of::<usize>()));
        fields.add_field_function_get("inner", |lua, this| {
            let inner = association::get(lua, CPTR_INNER, this)?
                .ok_or_else(|| LuaError::external("inner type not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr_info(methods);
        method_provider::provide_arr_info(methods);

        // ToString
        method_provider::provide_to_string(methods);
    }
}
