#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use libffi::low::ffi_abi_FFI_DEFAULT_ABI;
use libffi::middle::{Cif, Type};
use libffi::raw::ffi_get_struct_offsets;
use std::ptr;
use std::vec::Vec;

use crate::associate::{get_associate, set_associate};

use super::ctype::CType;

// pub fn ffi_get_struct_offsets(
//     abi: ffi_abi,
//     struct_type: *mut ffi_type,
//     offsets: *mut usize,
// ) -> ffi_status;

pub struct CStruct {
    libffi_cfi: Cif,
    libffi_type: Type,
    fields: Vec<Type>,
    offsets: Vec<usize>,
    size: usize,
}

const CSTRUCT_INNER: &str = "__cstruct_inner";

impl CStruct {
    pub fn new(fields: Vec<Type>) -> Self {
        let libffi_type = Type::structure(fields.clone());
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());
        let size = unsafe { (*libffi_type.as_raw_ptr()).size };
        let mut offsets = Vec::with_capacity(fields.len());
        for mut i in 0..fields.len() {
            dbg!(i);
            offsets.push(unsafe {
                ffi_get_struct_offsets(
                    ffi_abi_FFI_DEFAULT_ABI,
                    libffi_type.as_raw_ptr(),
                    ptr::from_mut(&mut i),
                ) as usize
            });
        }

        Self {
            libffi_cfi,
            libffi_type,
            fields,
            offsets,
            size,
        }
    }

    pub fn from_lua_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let len: usize = table.raw_len();
        let mut fields = Vec::with_capacity(len);

        for i in 0..len {
            // Test required
            let field_type: LuaAnyUserData = table.get(i + 1)?;
            fields.push(field_type.borrow::<CType>()?.get_type());
        }

        table.set_readonly(true);

        let cstruct = lua.create_userdata(Self::new(fields))?;
        set_associate(lua, CSTRUCT_INNER, cstruct.clone(), table)?;
        Ok(cstruct)
    }

    pub fn offset(&self, index: usize) -> usize {
        self.offsets[index]
    }
}

impl LuaUserData for CStruct {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_function_get("inner", |lua, this: LuaAnyUserData| {
            let table: LuaValue = get_associate(lua, CSTRUCT_INNER, this)?
                .ok_or(LuaError::external("inner field not found"))?;
            Ok(table)
        });
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index);
            Ok(offset)
        });
    }
}
