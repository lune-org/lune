#![allow(clippy::cargo_common_metadata)]

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use num::cast::{AsPrimitive, NumCast};
use std::marker::PhantomData;

use libffi::middle::Type;
use mlua::prelude::*;

use super::association_names::CTYPE_STATIC;
use super::c_arr::CArr;
use super::c_helper::get_ensured_size;
use super::c_ptr::CPtr;
use crate::ffi::ffi_association::set_association;
use crate::ffi::ffi_helper::get_ptr_from_userdata;

pub struct CType<T: ?Sized> {
    // for ffi_ptrarray_to_raw?
    // libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<&'static str>,
    signedness: bool,
    _phantom: PhantomData<T>,
}

// Static CType, for borrow, is operation
pub struct CTypeStatic {
    pub libffi_type: Type,
    pub size: usize,
    pub name: Option<&'static str>,
    pub signedness: bool,
}
impl CTypeStatic {
    fn new<T>(ctype: &CType<T>) -> Self {
        Self {
            libffi_type: ctype.libffi_type.clone(),
            size: ctype.size,
            name: ctype.name,
            signedness: ctype.signedness,
        }
    }
}
impl LuaUserData for CTypeStatic {}

impl<T> CType<T>
where
    Self: CTypeConvert,
    T: 'static,
{
    pub fn new_with_libffi_type<'lua>(
        lua: &'lua Lua,
        libffi_type: Type,
        signedness: bool,
        name: Option<&'static str>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        // let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;

        let ctype = Self {
            // libffi_cif: libffi_cfi,
            libffi_type,
            size,
            name,
            signedness,
            _phantom: PhantomData,
        };
        let userdata_static = lua.create_any_userdata(CTypeStatic::new::<T>(&ctype))?;
        let userdata = lua.create_userdata(ctype)?;

        set_association(lua, CTYPE_STATIC, &userdata, &userdata_static)?;

        Ok(userdata)
    }

    pub fn get_type(&self) -> &Type {
        &self.libffi_type
    }

    pub fn stringify(&self) -> &str {
        match self.name {
            Some(t) => t,
            None => "unnamed",
        }
    }

    pub fn cast_failed_with(&self, into_ctype: &LuaAnyUserData) -> LuaError {
        let config = ValueFormatConfig::new();
        LuaError::external(format!(
            "Cannot cast <CType({})> to {}",
            self.stringify(),
            pretty_format_value(&LuaValue::UserData(into_ctype.to_owned()), &config)
        ))
    }
}

// Handle C data, provide type conversion between luavalue and c-type
pub trait CTypeConvert {
    // Convert luavalue into data, then write into ptr
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()>;

    // Read data from ptr, then convert into luavalue
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue>;

    // Read data from userdata (such as box or ref) and convert it into luavalue
    unsafe fn read_userdata<'lua>(
        &self,
        lua: &'lua Lua,
        userdata: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaValue<'lua>> {
        let ptr = unsafe { get_ptr_from_userdata(&userdata, offset)? };
        let value = Self::ptr_into_luavalue(lua, ptr)?;
        Ok(value)
    }

    // Write data into userdata (such as box or ref) from luavalue
    unsafe fn write_userdata<'lua>(
        &self,
        luavalue: LuaValue<'lua>,
        userdata: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<()> {
        let ptr = unsafe { get_ptr_from_userdata(&userdata, offset)? };
        Self::luavalue_into_ptr(luavalue, ptr)?;
        Ok(())
    }
}

pub trait CTypeNumCast<T>
where
    T: NumCast,
{
    // Cast T as U
    fn cast_userdata<U>(from: &LuaAnyUserData, into: &LuaAnyUserData) -> LuaResult<()>
    where
        T: AsPrimitive<U>,
        U: 'static + Copy,
    {
        let from_ptr = unsafe { get_ptr_from_userdata(from, None)?.cast::<T>() };
        let into_ptr = unsafe { get_ptr_from_userdata(into, None)?.cast::<U>() };

        unsafe {
            *into_ptr = (*from_ptr).as_();
        }

        Ok(())
    }

    fn cast_userdata_if_type_match<U>(
        ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<Option<()>>
    where
        T: AsPrimitive<U>,
        U: 'static + Copy,
    {
        if ctype.is::<CType<U>>() {
            Self::cast_userdata(from, into)?;
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}

impl<T> LuaUserData for CType<T>
where
    Self: CTypeConvert,
    T: 'static,
{
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_meta_field(LuaMetaMethod::Type, "CType");
        fields.add_field_method_get("signedness", |_, this| Ok(this.signedness));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            CPtr::from_lua_userdata(lua, &this)
        });
        methods.add_method(
            "from",
            |lua, ctype, (userdata, offset): (LuaAnyUserData, Option<isize>)| unsafe {
                ctype.read_userdata(lua, userdata, offset)
            },
        );
        methods.add_method(
            "into",
            |_, ctype, (value, userdata, offset): (LuaValue, LuaAnyUserData, Option<isize>)| unsafe {
                ctype.write_userdata(value, userdata, offset)
            },
        );
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            CArr::from_lua_userdata(lua, &this, length)
        });
        methods.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            lua.create_string(this.stringify())
        });
    }
}
