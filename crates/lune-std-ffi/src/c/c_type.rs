#![allow(clippy::inline_always)]

use std::{cell::Ref, marker::PhantomData};

use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{CArr, CPtr};
use crate::ffi::{FfiBox, GetNativeData, NativeConvert, NativeData, NativeSignedness, NativeSize};
use crate::libffi_helper::get_ensured_size;

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

impl<T> NativeSize for CType<T> {
    fn get_size(&self) -> usize {
        self.size
    }
}

pub struct CType<T> {
    // for ffi_ptrarray_to_raw?
    // libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<&'static str>,
    _phantom: PhantomData<T>,
}
impl<T> CType<T>
where
    T: 'static,
    Self: CTypeCast + NativeSignedness + NativeConvert,
{
    pub fn new_with_libffi_type<'lua>(
        lua: &'lua Lua,
        libffi_type: Type,
        name: Option<&'static str>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;

        let ctype = Self {
            libffi_type,
            size,
            name,
            _phantom: PhantomData,
        };
        let userdata = lua.create_userdata(ctype)?;

        Ok(userdata)
    }

    pub fn stringify(&self) -> &str {
        match self.name {
            Some(t) => t,
            None => "unnamed",
        }
    }
}

impl<T> LuaUserData for CType<T>
where
    T: 'static,
    Self: CTypeCast + NativeSignedness + NativeConvert,
{
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.get_size()));
        fields.add_meta_field(LuaMetaMethod::Type, "CType");
        fields.add_field_method_get("signedness", |_, this| Ok(this.get_signedness()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            CPtr::new_from_lua_userdata(lua, &this)
        });
        methods.add_method("box", |lua, this, value: LuaValue| {
            let result = lua.create_userdata(FfiBox::new(this.get_size()))?;

            unsafe { this.luavalue_into(lua, 0, &result.get_data_handle()?, value)? };
            Ok(result)
        });
        methods.add_function(
            "from",
            |lua, (this, userdata, offset): (LuaAnyUserData, LuaAnyUserData, Option<isize>)| {
                let ctype = this.borrow::<Self>()?;
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, ctype.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_readable() {
                    return Err(LuaError::external("Unreadable data handle"));
                }

                unsafe { ctype.luavalue_from(lua, offset, data_handle) }
            },
        );
        methods.add_function(
            "into",
            |lua,
             (this, userdata, value, offset): (
                LuaAnyUserData,
                LuaAnyUserData,
                LuaValue,
                Option<isize>,
            )| {
                let ctype = this.borrow::<Self>()?;
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, ctype.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { ctype.luavalue_into(lua, offset, data_handle, value) }
            },
        );
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            CArr::new_from_lua_userdata(lua, &this, length)
        });
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
        methods.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            lua.create_string(this.stringify())
        });
    }
}
