use std::{cell::Ref, vec::Vec};

use libffi::{low, middle::Type, raw::ffi_get_struct_offsets};
use mlua::prelude::*;

use super::{association_names::CSTRUCT_INNER, helper, method_provider};
use crate::ffi::{
    association, libffi_helper::ffi_status_assert, FfiConvert, FfiData, FfiSignedness, FfiSize,
};

pub struct CStructInfo {
    middle_type: Type,
    size: usize,
    inner_offset_list: Vec<usize>,
    inner_conv_list: Vec<*const dyn FfiConvert>,
}

fn get_field_table<'lua>(
    lua: &'lua Lua,
    userdata: &LuaAnyUserData<'lua>,
) -> LuaResult<LuaTable<'lua>> {
    let value = association::get(lua, CSTRUCT_INNER, userdata)?
        .ok_or_else(|| LuaError::external("Failed to retrieve inner field table: not found"))?;
    if let LuaValue::Table(table) = value {
        Ok(table)
    } else {
        Err(LuaError::external(
            "Failed to retrieve inner field: not a table",
        ))
    }
}

impl CStructInfo {
    pub fn new(fields: Vec<Type>, inner_conv_list: Vec<*const dyn FfiConvert>) -> LuaResult<Self> {
        let len = fields.len();
        let mut inner_offset_list = Vec::<usize>::with_capacity(len);
        let middle_type = Type::structure(fields);

        // Get field offsets with ffi_get_struct_offsets
        unsafe {
            ffi_status_assert(ffi_get_struct_offsets(
                low::ffi_abi_FFI_DEFAULT_ABI,
                middle_type.as_raw_ptr(),
                inner_offset_list.as_mut_ptr(),
            ))?;
            inner_offset_list.set_len(len);
        }

        // Get tailing padded size of struct (get_ensured_size not required)
        let size = unsafe { (*middle_type.as_raw_ptr()).size };

        Ok(Self {
            middle_type,
            size,
            inner_offset_list,
            inner_conv_list,
        })
    }

    // Create new CStruct from LuaTable.
    // Freeze and hold table
    pub fn from_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        if helper::has_void(&table)? {
            return Err(LuaError::external("Void field in sturct is not allowed"));
        }

        let cstruct = lua
            .create_userdata(Self::new(helper::get_middle_type_list(&table)?, unsafe {
                helper::get_conv_list(&table)?
            })?)?;

        // Save field table
        table.set_readonly(true);
        association::set(lua, CSTRUCT_INNER, &cstruct, table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing
    // ex: <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let fields = get_field_table(lua, userdata)?;
        let mut stringified = String::from(" ");

        // Children
        for i in 0..fields.raw_len() {
            let child: LuaAnyUserData = fields.raw_get(i + 1)?;
            let pretty_formatted = helper::pretty_format(lua, &child)?;
            stringified.push_str(format!("{pretty_formatted}, ").as_str());
        }

        // Size
        stringified
            .push_str(format!("size = {} ", userdata.borrow::<CStructInfo>()?.get_size()).as_str());
        Ok(stringified)
    }

    // Get byte offset of nth field
    pub fn offset(&self, index: usize) -> LuaResult<usize> {
        Ok(self
            .inner_offset_list
            .get(index)
            .ok_or_else(|| LuaError::external("Out of index"))?
            .to_owned())
    }

    pub fn get_middle_type(&self) -> Type {
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
        for (index, conv) in self.inner_conv_list.iter().enumerate() {
            let field_offset = self.offset(index)? as isize;
            let data: LuaValue = table.get(index + 1)?;
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
    unsafe fn copy_data(
        &self,
        _lua: &Lua,
        dst_offset: isize,
        src_offset: isize,
        dst: &Ref<dyn FfiData>,
        src: &Ref<dyn FfiData>,
    ) -> LuaResult<()> {
        dst.get_inner_pointer().byte_offset(dst_offset).copy_from(
            src.get_inner_pointer().byte_offset(src_offset),
            self.get_size(),
        );
        Ok(())
    }
}

impl LuaUserData for CStructInfo {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_lua, this| Ok(this.get_size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_read_data(methods);
        method_provider::provide_write_data(methods);
        method_provider::provide_copy_data(methods);

        // Get nth field offset
        methods.add_method("offset", |_lua, this, index: usize| this.offset(index));
        // Get nth field type
        methods.add_function(
            "field",
            |lua, (this, field_index): (LuaAnyUserData, usize)| {
                let field_table = get_field_table(lua, &this)?;
                field_table.raw_get::<_, LuaAnyUserData>(field_index + 1)
            },
        );
    }
}
