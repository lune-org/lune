#![allow(clippy::cargo_common_metadata)]

use std::borrow::Borrow;

use super::association::{get_association, set_association};
use super::cstruct::CStruct;
use libffi::middle::{Cif, Type};
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;
// use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

const POINTER_INNER: &str = "__pointer_inner";

pub struct CType {
    libffi_cif: Cif,
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
            libffi_cif: libffi_cfi,
            libffi_type,
            size,
        }
    }

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }

    pub fn pointer<'lua>(lua: &'lua Lua, inner: LuaAnyUserData) -> LuaResult<LuaValue<'lua>> {
        let value = Self {
            libffi_cif: Cif::new(vec![Type::pointer()], Type::void()),
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
            let inner = get_association(lua, POINTER_INNER, this)?;
            match inner {
                Some(t) => Ok(t),
                None => Ok(LuaNil),
            }
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CType::pointer(lua, this)?;
            Ok(pointer)
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

pub fn libffi_types_from_table(table: &LuaTable) -> LuaResult<Vec<Type>> {
    let len: usize = table.raw_len();
    let mut fields = Vec::with_capacity(len);

    for i in 0..len {
        // Test required
        let value = table.raw_get(i + 1)?;
        match value {
            LuaValue::UserData(field_type) => {
                fields.push(libffi_type_from_userdata(&field_type)?);
            }
            _ => {
                return Err(LuaError::external(format!(
                    "Unexpected field. CStruct, CType or CArr is required for element but got {}",
                    pretty_format_value(&value, &ValueFormatConfig::new())
                )));
            }
        }
    }

    Ok(fields)
}

pub fn libffi_type_from_userdata(userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStruct>() {
        Ok(userdata.borrow::<CStruct>()?.get_type())
    } else if userdata.is::<CType>() {
        Ok(userdata.borrow::<CType>()?.get_type())
    } else {
        Err(LuaError::external(format!(
            "Unexpected field. CStruct, CType or CArr is required for element but got {}",
            pretty_format_value(
                // Since the data is in the Lua location,
                // there is no problem with the clone.
                &LuaValue::UserData(userdata.to_owned()),
                &ValueFormatConfig::new()
            )
        )))
    }
}
