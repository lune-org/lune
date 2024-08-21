#![allow(clippy::cargo_common_metadata)]

use std::borrow::Borrow;

use super::association::{get_association, set_association};
use libffi::middle::{Cif, Type};
use mlua::prelude::*;
// use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

const POINTER_INNER: &str = "__pointer_inner";

pub struct CType {
    libffi_cfi: Cif,
    libffi_type: Type,
    size: usize,
}

// TODO: ARR
// TODO: convert

impl CType {
    pub fn new(libffi_type: Type) -> Self {
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = unsafe { (*libffi_type.as_raw_ptr()).size };
        Self {
            libffi_cfi,
            libffi_type,
            size,
        }
    }

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }

    pub fn pointer<'lua>(lua: &'lua Lua, inner: LuaAnyUserData) -> LuaResult<LuaValue<'lua>> {
        let value = Self {
            libffi_cfi: Cif::new(vec![Type::pointer()], Type::void()),
            libffi_type: Type::pointer(),
            size: size_of::<usize>(),
        }
        .into_lua(lua)?;

        set_association(lua, POINTER_INNER, value.borrow(), inner)?;

        Ok(value)
    }
}

impl LuaUserData for CType {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_function_get("inner", |lua, this| {
            get_association(lua, POINTER_INNER, this)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            Ok(CType::pointer(lua, this))
        });
    }
}

pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    Ok(vec![
        ("u8", CType::new(Type::u8()).into_lua(lua)?),
        ("u16", CType::new(Type::u16()).into_lua(lua)?),
        ("u32", CType::new(Type::u32()).into_lua(lua)?),
        ("u64", CType::new(Type::u64()).into_lua(lua)?),
        ("i8", CType::new(Type::i8()).into_lua(lua)?),
        ("i16", CType::new(Type::i16()).into_lua(lua)?),
        ("i32", CType::new(Type::i32()).into_lua(lua)?),
        ("i64", CType::new(Type::i64()).into_lua(lua)?),
        ("f32", CType::new(Type::f32()).into_lua(lua)?),
        ("f64", CType::new(Type::f64()).into_lua(lua)?),
        ("void", CType::new(Type::void()).into_lua(lua)?),
    ])
}
