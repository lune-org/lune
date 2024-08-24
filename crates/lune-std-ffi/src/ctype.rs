#![allow(clippy::cargo_common_metadata)]

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use crate::carr::CArr;
use crate::chelper::get_ensured_size;
use crate::cptr::CPtr;
// use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

pub struct CType {
    libffi_cif: Cif,
    libffi_type: Type,
    size: usize,
    name: Option<String>,
}

impl CType {
    pub fn new(libffi_type: Type, name: Option<String>) -> LuaResult<Self> {
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = get_ensured_size(libffi_type.as_raw_ptr())?;
        Ok(Self {
            libffi_cif: libffi_cfi,
            libffi_type,
            size,
            name,
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
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    Ok(vec![
        (
            "u8",
            CType::new(Type::u8(), Some(String::from("u8")))?.into_lua(lua)?,
        ),
        (
            "u16",
            CType::new(Type::u16(), Some(String::from("u16")))?.into_lua(lua)?,
        ),
        (
            "u32",
            CType::new(Type::u32(), Some(String::from("u32")))?.into_lua(lua)?,
        ),
        (
            "u64",
            CType::new(Type::u64(), Some(String::from("u64")))?.into_lua(lua)?,
        ),
        (
            "i8",
            CType::new(Type::i8(), Some(String::from("i8")))?.into_lua(lua)?,
        ),
        (
            "i16",
            CType::new(Type::i16(), Some(String::from("i16")))?.into_lua(lua)?,
        ),
        (
            "i32",
            CType::new(Type::i32(), Some(String::from("i32")))?.into_lua(lua)?,
        ),
        (
            "i64",
            CType::new(Type::i64(), Some(String::from("i64")))?.into_lua(lua)?,
        ),
        (
            "f32",
            CType::new(Type::f32(), Some(String::from("f32")))?.into_lua(lua)?,
        ),
        (
            "f64",
            CType::new(Type::f64(), Some(String::from("f64")))?.into_lua(lua)?,
        ),
        (
            "void",
            CType::new(Type::void(), Some(String::from("void")))?.into_lua(lua)?,
        ),
    ])
}
