#![allow(clippy::cargo_common_metadata)]

use std::vec::Vec;

use libffi::{
    low,
    middle::{Cif, Type},
    raw,
};
use mlua::prelude::*;

use super::association_names::CSTRUCT_INNER;
use super::c_arr::CArr;
use super::c_helper::{name_from_userdata, stringify_userdata, type_list_from_table};
use super::c_ptr::CPtr;
use super::c_type::CType;
use crate::ffi::ffi_association::{get_association, set_association};
use crate::ffi::ffi_helper::FFI_STATUS_NAMES;

pub struct CStruct {
    libffi_cif: Cif,
    libffi_type: Type,
    fields: Vec<Type>,
    offsets: Vec<usize>,
    size: usize,
}

impl CStruct {
    pub fn new(fields: Vec<Type>) -> LuaResult<Self> {
        let libffi_type = Type::structure(fields.clone());
        let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());

        // Get field offsets with ffi_get_struct_offsets
        let mut offsets = Vec::<usize>::with_capacity(fields.len());
        unsafe {
            let offset_result: raw::ffi_status = raw::ffi_get_struct_offsets(
                low::ffi_abi_FFI_DEFAULT_ABI,
                libffi_type.as_raw_ptr(),
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
        let size = unsafe { (*libffi_type.as_raw_ptr()).size };

        Ok(Self {
            libffi_cif: libffi_cfi,
            libffi_type,
            fields,
            offsets,
            size,
        })
    }

    // Create new CStruct UserData with LuaTable.
    // Lock and hold table for .inner ref
    pub fn from_lua_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = type_list_from_table(&table)?;
        let cstruct = lua.create_userdata(Self::new(fields)?)?;
        table.set_readonly(true);
        set_association(lua, CSTRUCT_INNER, cstruct.clone(), table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let field: LuaValue = userdata.get("inner")?;
        if field.is_table() {
            let table = field
                .as_table()
                .ok_or(LuaError::external("failed to get inner type table."))?;
            // iterate for field
            let mut result = String::from(" ");
            for i in 0..table.raw_len() {
                let child: LuaAnyUserData = table.raw_get(i + 1)?;
                if child.is::<CType>() {
                    result.push_str(format!("{}, ", stringify_userdata(&child)?).as_str());
                } else {
                    result.push_str(
                        format!(
                            "<{}({})>, ",
                            name_from_userdata(&child),
                            stringify_userdata(&child)?
                        )
                        .as_str(),
                    );
                }
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

    pub fn get_type(&self) -> Type {
        self.libffi_type.clone()
    }
}

impl LuaUserData for CStruct {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));

        // Simply pass in the locked table used when first creating this object.
        // By strongly referencing the table, the types inside do not disappear
        // and the user can read the contents as needed. (good recycling!)
        fields.add_field_function_get("inner", |lua, this: LuaAnyUserData| {
            let table: LuaValue = get_association(lua, CSTRUCT_INNER, this)?
                // It shouldn't happen.
                .ok_or(LuaError::external("inner field not found"))?;
            Ok(table)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index)?;
            Ok(offset)
        });
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            let carr = CArr::from_lua_userdata(lua, &this, length)?;
            Ok(carr)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let result = CStruct::stringify(&this)?;
            Ok(result)
        });
    }
}
