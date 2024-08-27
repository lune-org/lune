#![allow(clippy::cargo_common_metadata)]

use std::vec::Vec;

use libffi::{low, middle::Type, raw};
use mlua::prelude::*;

use super::association_names::CSTRUCT_INNER;
use super::c_arr::CArr;
use super::c_helper::{pretty_format_userdata, type_list_from_table};
use super::c_ptr::CPtr;
use crate::ffi::ffi_association::{get_association, set_association};
use crate::ffi::ffi_helper::FFI_STATUS_NAMES;

pub struct CStruct {
    // libffi_cif: Cif,
    fields: Vec<Type>,
    struct_type: Type,
    offsets: Vec<usize>,
    size: usize,
}

impl CStruct {
    pub fn new(fields: Vec<Type>) -> LuaResult<Self> {
        let struct_type = Type::structure(fields.iter().cloned());
        // let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());

        // Get field offsets with ffi_get_struct_offsets
        let mut offsets = Vec::<usize>::with_capacity(fields.len());
        unsafe {
            let offset_result: raw::ffi_status = raw::ffi_get_struct_offsets(
                low::ffi_abi_FFI_DEFAULT_ABI,
                struct_type.as_raw_ptr(),
                offsets.as_mut_ptr(),
            );
            if offset_result != raw::ffi_status_FFI_OK {
                return Err(LuaError::external(format!(
                    "ffi_get_struct_offsets failed. expected result {}, got {}",
                    FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[offset_result as usize]
                )));
            }
            offsets.set_len(offsets.capacity());
        }

        // Get tailing padded size of struct
        // See http://www.chiark.greenend.org.uk/doc/libffi-dev/html/Size-and-Alignment.html
        let size = unsafe { (*struct_type.as_raw_ptr()).size };

        Ok(Self {
            // libffi_cif: libffi_cfi,
            fields,
            struct_type,
            offsets,
            size,
        })
    }

    // Create new CStruct UserData with LuaTable.
    // Lock and hold table for .inner ref
    pub fn new_from_lua_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = type_list_from_table(lua, &table)?;
        let cstruct = lua.create_userdata(Self::new(fields)?)?;
        table.set_readonly(true);
        set_association(lua, CSTRUCT_INNER, &cstruct, table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let field: LuaValue = userdata.get("inner")?;
        if field.is_table() {
            let table = field
                .as_table()
                .ok_or(LuaError::external("failed to get inner type table."))?;
            // iterate for field
            let mut result = String::from(" ");
            for i in 0..table.raw_len() {
                let child: LuaAnyUserData = table.raw_get(i + 1)?;
                result.push_str(pretty_format_userdata(lua, &child)?.as_str());
            }

            // size of
            result.push_str(format!("size = {} ", userdata.borrow::<CStruct>()?.size).as_str());
            Ok(result)
        } else {
            Err(LuaError::external("failed to get inner type table."))
        }
    }

    // Get byte offset of nth field
    pub fn offset(&self, index: usize) -> LuaResult<usize> {
        let offset = self
            .offsets
            .get(index)
            .ok_or(LuaError::external("Out of index"))?
            .to_owned();
        Ok(offset)
    }

    pub fn get_fields(&self) -> &Vec<Type> {
        &self.fields
    }

    pub fn get_type(&self) -> &Type {
        &self.struct_type
    }
}

impl LuaUserData for CStruct {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index)?;
            Ok(offset)
        });
        // Simply pass type in the locked table used when first creating this object.
        // By referencing the table to struct, the types inside do not disappear
        methods.add_function("field", |lua, (this, field): (LuaAnyUserData, usize)| {
            if let LuaValue::Table(t) = get_association(lua, CSTRUCT_INNER, this)?
                .ok_or(LuaError::external("Field table not found"))?
            {
                let value: LuaValue = t.get(field + 1)?;
                Ok(value)
            } else {
                Err(LuaError::external("Failed to read field table"))
            }
        });
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            let carr = CArr::new_from_lua_userdata(lua, &this, length)?;
            Ok(carr)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |lua, this: LuaAnyUserData| {
            let result = CStruct::stringify(lua, &this)?;
            Ok(result)
        });
    }
}
