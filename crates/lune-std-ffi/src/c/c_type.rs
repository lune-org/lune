#![allow(clippy::cargo_common_metadata)]

use core::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort, c_void,
};

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::c_arr::CArr;
use super::c_helper::get_ensured_size;
use super::c_ptr::CPtr;
use crate::ffi::ffi_helper::get_ptr_from_userdata;
use crate::ffi::ffi_platform::CHAR_IS_SIGNED;
// use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

pub struct CType {
    libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<String>,

    // Write converted data from luavalue into some ptr
    pub luavalue_into_ptr: fn(value: LuaValue, ptr: *mut c_void) -> LuaResult<()>,

    // Read luavalue from some ptr
    pub ptr_into_luavalue: fn(lua: &Lua, ptr: *mut c_void) -> LuaResult<LuaValue>,
}

impl CType {
    pub fn new(
        libffi_type: Type,
        name: Option<String>,
        luavalue_into_ptr: fn(value: LuaValue, ptr: *mut c_void) -> LuaResult<()>,
        ptr_into_luavalue: fn(lua: &Lua, ptr: *mut c_void) -> LuaResult<LuaValue>,
    ) -> LuaResult<Self> {
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;
        Ok(Self {
            libffi_cif: libffi_cfi,
            libffi_type,
            size,
            name,
            luavalue_into_ptr,
            ptr_into_luavalue,
        })
    }

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }

    pub fn stringify(&self) -> String {
        match &self.name {
            Some(t) => t.to_owned(),
            None => String::from("unnamed"),
        }
    }

    // Read data from ptr and convert it into luavalue
    pub unsafe fn read_ptr<'lua>(
        &self,
        lua: &'lua Lua,
        userdata: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaValue<'lua>> {
        let ptr = unsafe { get_ptr_from_userdata(&userdata, offset)? };
        let value = (self.ptr_into_luavalue)(lua, ptr)?;
        Ok(value)
    }

    // Write converted data from luavalue into ptr
    pub unsafe fn write_ptr<'lua>(
        &self,
        luavalue: LuaValue<'lua>,
        userdata: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<()> {
        let ptr = unsafe { get_ptr_from_userdata(&userdata, offset)? };
        (self.luavalue_into_ptr)(luavalue, ptr)?;
        Ok(())
    }
}

impl LuaUserData for CType {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_method(
            "from",
            |lua, ctype, (userdata, offset): (LuaAnyUserData, Option<isize>)| {
                let value = unsafe { ctype.read_ptr(lua, userdata, offset)? };
                Ok(value)
            },
        );
        methods.add_method(
            "into",
            |_, ctype, (value, userdata, offset): (LuaValue, LuaAnyUserData, Option<isize>)| {
                unsafe { ctype.write_ptr(value, userdata, offset)? };
                Ok(())
            },
        );
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            let carr = CArr::from_lua_userdata(lua, &this, length)?;
            Ok(carr)
        });
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            let name = this.stringify();
            Ok(name)
        });
    }
}

// export all default c-types
#[allow(clippy::too_many_lines)]
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    Ok(vec![
        (
            "int",
            CType::new(
                Type::c_int(),
                Some(String::from("int")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_int;
                    unsafe {
                        *(ptr.cast::<c_int>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut c_void| {
                    let value = unsafe { (*ptr.cast::<c_int>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "long",
            CType::new(
                Type::c_long(),
                Some(String::from("long")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_long;
                    unsafe {
                        *(ptr.cast::<c_long>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut c_void| {
                    let value = unsafe { (*ptr.cast::<c_long>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "longlong",
            CType::new(
                Type::c_longlong(),
                Some(String::from("longlong")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_longlong;
                    unsafe {
                        *(ptr.cast::<c_longlong>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut c_void| {
                    let value = unsafe { (*ptr.cast::<c_longlong>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "char",
            CType::new(
                if CHAR_IS_SIGNED {
                    Type::c_schar()
                } else {
                    Type::c_uchar()
                },
                Some(String::from("char")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_char;
                    unsafe {
                        *(ptr.cast::<c_char>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut c_void| {
                    let value = unsafe { (*ptr.cast::<c_char>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
    ])
}
