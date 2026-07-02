#![allow(clippy::inline_always)]

use std::marker::PhantomData;

use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{helper, method_provider};
use crate::{
    data::GetFfiData,
    ffi::{libffi_helper::get_ensured_size, FfiConvert, FfiData, FfiSignedness, FfiSize},
};

// Provide type casting
// This trait should be implemented for each types
pub trait CTypeCast {
    #[inline(always)]
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        _from: &dyn FfiData,
        _into: &dyn FfiData,
        _from_offset: isize,
        _into_offset: isize,
    ) -> LuaResult<()> {
        // Error if have no cast implement
        Err(Self::cast_failed_with(self, from_ctype, into_ctype))
    }

    fn cast_failed_with(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
    ) -> LuaError {
        let config = ValueFormatConfig::new();
        LuaError::external(format!(
            "Failed to cast {} into {}",
            pretty_format_value(&LuaValue::UserData(from_ctype.to_owned()), &config),
            pretty_format_value(&LuaValue::UserData(into_ctype.to_owned()), &config),
        ))
    }
}

pub struct CTypeInfo<T> {
    middle_type: Type,
    size: usize,
    name: &'static str,
    _phantom: PhantomData<T>,
}

impl<T> FfiSize for CTypeInfo<T> {
    fn get_size(&self) -> usize {
        self.size
    }
}

impl<T> CTypeInfo<T>
where
    T: 'static,
    Self: CTypeCast + FfiSignedness + FfiConvert + FfiSize,
{
    pub fn from_middle_type(
        lua: &Lua,
        libffi_type: Type,
        name: &'static str,
    ) -> LuaResult<LuaAnyUserData> {
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

    pub fn get_middle_type(&self) -> Type {
        self.middle_type.clone()
    }
}

impl<T> LuaUserData for CTypeInfo<T>
where
    T: 'static,
    Self: CTypeCast + FfiSignedness + FfiConvert + FfiSize,
{
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field(LuaMetaMethod::Type, "CTypeInfo");
        fields.add_field_method_get("size", |_lua, this| Ok(this.get_size()));
        fields.add_field_method_get("signedness", |_lua, this| Ok(this.get_signedness()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_read_data(methods);
        method_provider::provide_write_data(methods);
        method_provider::provide_copy_data(methods);
        method_provider::provide_stringify_data(methods);

        // Math
        // TODO: arithmetic methods, once Lune has a Luau with 64-bit integers
        // (f64 can't represent i64/u64/i128/u128 losslessly)

        methods.add_function(
            "cast",
            |_lua,
             (from_type, into_type, from, into, from_offset, into_offset): (
                LuaAnyUserData,
                LuaAnyUserData,
                LuaAnyUserData,
                LuaAnyUserData,
                Option<isize>,
                Option<isize>,
            )| {
                let from_offset = from_offset.unwrap_or(0);
                let into_offset = into_offset.unwrap_or(0);

                let from_data = from.get_ffi_data()?;
                let into_data = into.get_ffi_data()?;

                if !from_data.check_inner_boundary(from_offset, from_type.borrow::<Self>()?.get_size())
                {
                    return Err(LuaError::external("Source out of bounds"));
                }
                if !from_data.is_readable() {
                    return Err(LuaError::external("Source is not readable"));
                }
                if !into_data.check_inner_boundary(into_offset, helper::get_size(&into_type)?) {
                    return Err(LuaError::external("Destination out of bounds"));
                }
                if !into_data.is_writable() {
                    return Err(LuaError::external("Destination is not writable"));
                }

                from_type.borrow::<Self>()?.cast(
                    &from_type,
                    &into_type,
                    &from_data,
                    &into_data,
                    from_offset,
                    into_offset,
                )
            },
        );
    }
}
