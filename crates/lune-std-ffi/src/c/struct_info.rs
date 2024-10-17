use std::{cell::Ref, vec::Vec};

use libffi::{low, middle::Type, raw};
use mlua::prelude::*;

use super::{association_names::CSTRUCT_INNER, helper, method_provider};
use crate::ffi::{
    association, libffi_helper::FFI_STATUS_NAMES, FfiConvert, FfiData, FfiSignedness, FfiSize,
};

pub struct CStructInfo {
    middle_type: Type,
    size: usize,
    inner_offset_list: Vec<usize>,
    inner_conv_list: Vec<*const dyn FfiConvert>,
}

impl CStructInfo {
    pub fn new(fields: Vec<Type>, inner_conv_list: Vec<*const dyn FfiConvert>) -> LuaResult<Self> {
        let len = fields.len();
        let mut inner_offset_list = Vec::<usize>::with_capacity(len);
        let middle_type = Type::structure(fields);

        // Get field offsets with ffi_get_struct_offsets
        unsafe {
            let offset_result: raw::ffi_status = raw::ffi_get_struct_offsets(
                low::ffi_abi_FFI_DEFAULT_ABI,
                middle_type.as_raw_ptr(),
                inner_offset_list.as_mut_ptr(),
            );
            if offset_result != raw::ffi_status_FFI_OK {
                return Err(LuaError::external(format!(
                    "ffi_get_struct_offsets failed. expected result {}, got {}",
                    FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[offset_result as usize]
                )));
            }
            inner_offset_list.set_len(len);
        }

        // Get tailing padded size of struct
        // See http://www.chiark.greenend.org.uk/doc/libffi-dev/html/Size-and-Alignment.html
        // In here, using get_ensured_size is not required
        let size = unsafe { (*middle_type.as_raw_ptr()).size };

        Ok(Self {
            middle_type,
            size,
            inner_offset_list,
            inner_conv_list,
        })
    }

    // Create new CStruct UserData with LuaTable.
    // Lock and hold table for .inner ref
    pub fn from_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let cstruct = lua
            .create_userdata(Self::new(helper::get_middle_type_list(&table)?, unsafe {
                helper::get_conv_list(&table)?
            })?)?;

        table.set_readonly(true);
        association::set(lua, CSTRUCT_INNER, &cstruct, table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::Table(fields) = association::get(lua, CSTRUCT_INNER, userdata)?
            .ok_or_else(|| LuaError::external("Field table not found"))?
        {
            let mut result = String::from(" ");
            for i in 0..fields.raw_len() {
                let child: LuaAnyUserData = fields.raw_get(i + 1)?;
                let pretty_formatted = helper::pretty_format(lua, &child)?;
                result.push_str(format!("{pretty_formatted}, ").as_str());
            }

            // size of
            result.push_str(
                format!("size = {} ", userdata.borrow::<CStructInfo>()?.get_size()).as_str(),
            );
            Ok(result)
        } else {
            Err(LuaError::external("failed to get inner type table."))
        }
    }

    // Get byte offset of nth field
    pub fn offset(&self, index: usize) -> LuaResult<usize> {
        let offset = self
            .inner_offset_list
            .get(index)
            .ok_or_else(|| LuaError::external("Out of index"))?
            .to_owned();
        Ok(offset)
    }

    pub fn get_type(&self) -> Type {
        self.middle_type.clone()
    }
}

impl FfiSize for CStructInfo {
    fn get_size(&self) -> usize {
        self.size
    }
}
impl FfiSignedness for CStructInfo {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl FfiConvert for CStructInfo {
    // FIXME: FfiBox, FfiRef support required
    unsafe fn value_into_data<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let LuaValue::Table(ref table) = value else {
            return Err(LuaError::external("Value is not a table"));
        };
        for (i, conv) in self.inner_conv_list.iter().enumerate() {
            let field_offset = self.offset(i)? as isize;
            let data: LuaValue = table.get(i + 1)?;

            conv.as_ref().unwrap().value_into_data(
                lua,
                field_offset + offset,
                data_handle,
                data,
            )?;
        }
        Ok(())
    }

    unsafe fn value_from_data<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(self.inner_conv_list.len(), 0)?;
        for (i, conv) in self.inner_conv_list.iter().enumerate() {
            let field_offset = self.offset(i)? as isize;
            table.set(
                i + 1,
                conv.as_ref()
                    .unwrap()
                    .value_from_data(lua, field_offset + offset, data_handle)?,
            )?;
        }
        Ok(LuaValue::Table(table))
    }
}

impl LuaUserData for CStructInfo {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.get_size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr_info(methods);
        method_provider::provide_arr_info(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_from_data(methods);
        method_provider::provide_into_data(methods);

        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index)?;
            Ok(offset)
        });
        // Simply pass type in the locked table used when first creating this object.
        // By referencing the table to struct, the types inside do not disappear
        methods.add_function("field", |lua, (this, field): (LuaAnyUserData, usize)| {
            if let LuaValue::Table(fields) = association::get(lua, CSTRUCT_INNER, this)?
                .ok_or_else(|| LuaError::external("Field table not found"))?
            {
                let value: LuaValue = fields.raw_get(field + 1)?;
                Ok(value)
            } else {
                Err(LuaError::external("Failed to read field table"))
            }
        });
    }
}
