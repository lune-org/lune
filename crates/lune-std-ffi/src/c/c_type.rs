#![allow(clippy::cargo_common_metadata)]

use libffi::middle::Type;
use mlua::prelude::*;

use super::c_arr::CArr;
use super::c_helper::get_ensured_size;
use super::c_ptr::CPtr;
use crate::ffi::ffi_helper::get_ptr_from_userdata;

pub struct CType {
    // for ffi_ptrarray_to_raw?
    // libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<String>,

    // Write converted data from luavalue into some ptr
    pub luavalue_into_ptr: fn(value: LuaValue, ptr: *mut ()) -> LuaResult<()>,

    // Read luavalue from some ptr
    pub ptr_into_luavalue: fn(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue>,
}

impl CType {
    pub fn new(
        libffi_type: Type,
        name: Option<String>,
        luavalue_into_ptr: fn(value: LuaValue, ptr: *mut ()) -> LuaResult<()>,
        ptr_into_luavalue: fn(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue>,
    ) -> LuaResult<Self> {
        // let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;
        Ok(Self {
            // libffi_cif: libffi_cfi,
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
