use std::{cell::Ref, vec::Vec};

use libffi::{low, middle::Type, raw};
use mlua::prelude::*;

use super::{association_names::CSTRUCT_INNER, c_helper, method_provider, CArr, CPtr};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    NativeConvert, NativeData, NativeSignedness, NativeSize, FFI_STATUS_NAMES,
};

pub struct CStruct {
    middle_type: Type,
    size: usize,
    inner_offset_list: Vec<usize>,
    inner_conv_list: Vec<*const dyn NativeConvert>,
}

impl CStruct {
    pub fn new(
        fields: Vec<Type>,
        inner_conv_list: Vec<*const dyn NativeConvert>,
    ) -> LuaResult<Self> {
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
    pub fn new_from_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let cstruct = lua.create_userdata(Self::new(
            c_helper::get_middle_type_list(&table)?,
            unsafe { c_helper::get_conv_list(&table)? },
        )?)?;

        table.set_readonly(true);
        set_association(lua, CSTRUCT_INNER, &cstruct, table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::Table(fields) = get_association(lua, CSTRUCT_INNER, userdata)?
            .ok_or(LuaError::external("Field table not found"))?
        {
            let mut result = String::from(" ");
            for i in 0..fields.raw_len() {
                let child: LuaAnyUserData = fields.raw_get(i + 1)?;
                let pretty_formatted = c_helper::pretty_format(lua, &child)?;
                result.push_str(format!("{pretty_formatted}, ").as_str());
            }

            // size of
            result
                .push_str(format!("size = {} ", userdata.borrow::<CStruct>()?.get_size()).as_str());
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
            .ok_or(LuaError::external("Out of index"))?
            .to_owned();
        Ok(offset)
    }

    pub fn get_type(&self) -> Type {
        self.middle_type.clone()
    }
}

impl NativeSize for CStruct {
    fn get_size(&self) -> usize {
        self.size
    }
}
impl NativeSignedness for CStruct {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl NativeConvert for CStruct {
    // FIXME: FfiBox, FfiRef support required
    unsafe fn luavalue_into<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let LuaValue::Table(ref table) = value else {
            return Err(LuaError::external("Value is not a table"));
        };
        for (i, conv) in self.inner_conv_list.iter().enumerate() {
            let field_offset = self.offset(i)? as isize;
            let data: LuaValue = table.get(i + 1)?;

            conv.as_ref()
                .unwrap()
                .luavalue_into(lua, field_offset + offset, data_handle, data)?;
        }
        Ok(())
    }

    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(self.inner_conv_list.len(), 0)?;
        for (i, conv) in self.inner_conv_list.iter().enumerate() {
            let field_offset = self.offset(i)? as isize;
            table.set(
                i + 1,
                conv.as_ref()
                    .unwrap()
                    .luavalue_from(lua, field_offset + offset, data_handle)?,
            )?;
        }
        Ok(LuaValue::Table(table))
    }
}

impl LuaUserData for CStruct {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.get_size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_from(methods);
        method_provider::provide_into(methods);

        methods.add_method("offset", |_, this, index: usize| {
            let offset = this.offset(index)?;
            Ok(offset)
        });
        // Simply pass type in the locked table used when first creating this object.
        // By referencing the table to struct, the types inside do not disappear
        methods.add_function("field", |lua, (this, field): (LuaAnyUserData, usize)| {
            if let LuaValue::Table(fields) = get_association(lua, CSTRUCT_INNER, this)?
                .ok_or(LuaError::external("Field table not found"))?
            {
                let value: LuaValue = fields.raw_get(field + 1)?;
                Ok(value)
            } else {
                Err(LuaError::external("Failed to read field table"))
            }
        });
    }
}
