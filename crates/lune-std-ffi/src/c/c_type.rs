#![allow(clippy::inline_always)]

use std::{cell::Ref, marker::PhantomData};

use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use crate::{
    c::method_provider,
    ffi::{GetNativeData, NativeConvert, NativeData, NativeSignedness, NativeSize},
    libffi_helper::get_ensured_size,
};

// Cast native data
pub trait CTypeCast {
    #[inline(always)]
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        _from: &Ref<dyn NativeData>,
        _into: &Ref<dyn NativeData>,
    ) -> LuaResult<()> {
        // Show error if have no cast implement
        Err(Self::cast_failed_with(self, from_ctype, into_ctype))
    }

    fn cast_failed_with(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
    ) -> LuaError {
        let config = ValueFormatConfig::new();
        LuaError::external(format!(
            "Cannot cast {} to {}",
            pretty_format_value(&LuaValue::UserData(from_ctype.to_owned()), &config),
            pretty_format_value(&LuaValue::UserData(into_ctype.to_owned()), &config),
        ))
    }
}

pub struct CType<T> {
    middle_type: Type,
    size: usize,
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> NativeSize for CType<T> {
    fn get_size(&self) -> usize {
        self.size
    }
}

impl<T> CType<T>
where
    T: 'static,
    Self: CTypeCast + NativeSignedness + NativeConvert + NativeSize,
{
    pub fn new_with_libffi_type<'lua>(
        lua: &'lua Lua,
        libffi_type: Type,
        name: &'static str,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;

        let ctype = Self {
            middle_type: libffi_type,
            size,
            name,
            _phantom: PhantomData,
        };
        let userdata = lua.create_userdata(ctype)?;

        Ok(userdata)
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_type(&self) -> Type {
        self.middle_type.clone()
    }
}

impl<T> LuaUserData for CType<T>
where
    T: 'static,
    Self: CTypeCast + NativeSignedness + NativeConvert + NativeSize,
{
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.get_size()));
        fields.add_meta_field(LuaMetaMethod::Type, "CType");
        fields.add_field_method_get("signedness", |_, this| Ok(this.get_signedness()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_from(methods);
        method_provider::provide_into(methods);

        methods.add_function(
            "cast",
            |_,
             (from_type, into_type, from, into): (
                LuaAnyUserData,
                LuaAnyUserData,
                LuaAnyUserData,
                LuaAnyUserData,
            )| {
                from_type.borrow::<Self>()?.cast(
                    &from_type,
                    &into_type,
                    &from.get_data_handle()?,
                    &into.get_data_handle()?,
                )
            },
        );
    }
}
