use std::{cell::Ref, vec::Vec};

use libffi::{low, middle::Type, raw};
use mlua::prelude::*;

use super::{
    association_names::CSTRUCT_INNER,
    c_helper::{get_conv_list_from_table, libffi_type_list_from_table, pretty_format_userdata},
    CArr, CPtr,
};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    FfiBox, GetNativeData, NativeConvert, NativeData, NativeSignedness, NativeSize,
    FFI_STATUS_NAMES,
};

pub struct CStruct {
    // libffi_cif: Cif,
    // fields: Vec<Type>,
    struct_type: Type,
    offsets: Vec<usize>,
    size: usize,
    conv: Vec<*const dyn NativeConvert>,
}

impl CStruct {
    pub fn new(fields: Vec<Type>, conv: Vec<*const dyn NativeConvert>) -> LuaResult<Self> {
        let len = fields.len();
        let mut offsets = Vec::<usize>::with_capacity(len);
        let struct_type = Type::structure(fields);
        // let struct_type = Type::structure(fields.iter().cloned());
        // let libffi_cfi = Cif::new(vec![libffi_type.clone()], Type::void());

        // Get field offsets with ffi_get_struct_offsets
        // let mut offsets = Vec::<usize>::with_capacity(fields.len());
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
        // In here, using get_ensured_size is waste
        let size = unsafe { (*struct_type.as_raw_ptr()).size };

        Ok(Self {
            // libffi_cif: libffi_cfi,
            // fields,
            struct_type,
            offsets,
            size,
            conv,
        })
    }

    // Create new CStruct UserData with LuaTable.
    // Lock and hold table for .inner ref
    pub fn new_from_lua_table<'lua>(
        lua: &'lua Lua,
        table: LuaTable<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let cstruct = lua.create_userdata(Self::new(
            libffi_type_list_from_table(lua, &table)?,
            unsafe { get_conv_list_from_table(&table)? },
        )?)?;

        table.set_readonly(true);
        set_association(lua, CSTRUCT_INNER, &cstruct, table)?;
        Ok(cstruct)
    }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::Table(fields) = get_association(lua, CSTRUCT_INNER, userdata)?
            .ok_or(LuaError::external("Field table not found"))?
        {
            let mut result = String::from(" ");
            for i in 0..fields.raw_len() {
                let child: LuaAnyUserData = fields.raw_get(i + 1)?;
                result.push_str(pretty_format_userdata(lua, &child)?.as_str());
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
            .offsets
            .get(index)
            .ok_or(LuaError::external("Out of index"))?
            .to_owned();
        Ok(offset)
    }

    // pub fn get_fields(&self) -> &Vec<Type> {
    //     &self.fields
    // }

    pub fn get_type(&self) -> &Type {
        &self.struct_type
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
        for (i, conv) in self.conv.iter().enumerate() {
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
        let table = lua.create_table_with_capacity(self.conv.len(), 0)?;
        for (i, conv) in self.conv.iter().enumerate() {
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
        methods.add_method("box", |lua, this, table: LuaValue| {
            let result = lua.create_userdata(FfiBox::new(this.get_size()))?;

            unsafe { this.luavalue_into(lua, 0, &result.get_data_handle()?, table)? };
            Ok(result)
        });
        methods.add_method(
            "from",
            |lua, this, (userdata, offset): (LuaAnyUserData, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.check_readable(offset, this.get_size()) {
                    return Err(LuaError::external("Unreadable data handle"));
                }

                unsafe { this.luavalue_from(lua, offset, data_handle) }
            },
        );
        methods.add_method(
            "into",
            |lua, this, (userdata, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.checek_writable(offset, this.get_size()) {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { this.luavalue_into(lua, offset, data_handle, value) }
            },
        );
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::new_from_lua_userdata(lua, &this)?;
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
