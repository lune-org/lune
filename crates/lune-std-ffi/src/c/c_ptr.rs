use std::cell::Ref;

use libffi::middle::Type;
use mlua::prelude::*;

use super::{association_names::CPTR_INNER, c_helper, c_type_helper, method_provider};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    NativeConvert, NativeData, NativeSignedness, NativeSize,
};

pub struct CPtr();

impl NativeSignedness for CPtr {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl NativeSize for CPtr {
    fn get_size(&self) -> usize {
        size_of::<*mut ()>()
    }
}
impl NativeConvert for CPtr {
    // Convert luavalue into data, then write into ptr
    unsafe fn luavalue_into<'lua>(
        &self,
        _lua: &'lua Lua,
        _offset: isize,
        _data_handle: &Ref<dyn NativeData>,
        _value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        Err(LuaError::external("Conversion of pointer is not allowed"))
    }

    // Read data from ptr, then convert into luavalue
    unsafe fn luavalue_from<'lua>(
        &self,
        _lua: &'lua Lua,
        _offset: isize,
        _data_handle: &Ref<dyn NativeData>,
    ) -> LuaResult<LuaValue<'lua>> {
        Err(LuaError::external("Conversion of pointer is not allowed"))
    }
}

impl CPtr {
    // Create pointer type with '.inner' field
    // inner can be CArr, CType or CStruct
    pub fn from_userdata<'lua>(
        lua: &'lua Lua,
        inner: &LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let value = lua.create_userdata(Self())?;

        set_association(lua, CPTR_INNER, &value, inner)?;

        Ok(value)
    }

    // Stringify CPtr with inner ctype
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::UserData(inner_userdata) = userdata.get("inner")? {
            let pretty_formatted = c_helper::pretty_format(lua, &inner_userdata)?;
            Ok(if c_type_helper::is_ctype(&inner_userdata) {
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

impl LuaUserData for CPtr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, _| Ok(size_of::<usize>()));
        fields.add_field_function_get("inner", |lua, this| {
            let inner = get_association(lua, CPTR_INNER, this)?
                .ok_or(LuaError::external("inner type not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);
    }
}
