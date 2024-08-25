#![allow(clippy::cargo_common_metadata)]

use std::marker::PhantomData;

use libffi::middle::Type;
use mlua::prelude::*;

use super::c_arr::CArr;
use super::c_helper::get_ensured_size;
use super::c_ptr::CPtr;
use crate::ffi::ffi_helper::get_ptr_from_userdata;

pub struct CType<T: ?Sized> {
    // for ffi_ptrarray_to_raw?
    // libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<&'static str>,
    _phantom: PhantomData<T>,
}

impl<T> CType<T>
where
    T: ?Sized,
{
    pub fn new_with_libffi_type(libffi_type: Type, name: Option<&'static str>) -> LuaResult<Self> {
        // let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;
        Ok(Self {
            // libffi_cif: libffi_cfi,
            libffi_type,
            size,
            name,
            _phantom: PhantomData {},
        })
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
}

pub trait PtrHandle {
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

impl<T> LuaUserData for CType<T>
where
    Self: Sized + PtrHandle,
{
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
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
