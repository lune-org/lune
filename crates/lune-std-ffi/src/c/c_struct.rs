use std::{cell::Ref, vec::Vec};

use libffi::{low, middle::Type, raw};
use mlua::prelude::*;

use super::{
    association_names::CSTRUCT_INNER,
    c_helper::{
        libffi_type_list_from_table, luavalue_into_ptr, pretty_format_userdata, ptr_into_luavalue,
        type_size_from_userdata,
    },
    CArr, CPtr,
};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    ffi_helper::FFI_STATUS_NAMES,
    FfiBox, FfiRef, NativeConvert, NativeDataHandle, NativeSized,
};

pub struct CStruct {
    // libffi_cif: Cif,
    // fields: Vec<Type>,
    struct_type: Type,
    offsets: Vec<usize>,
    size: usize,
}

impl CStruct {
    pub fn new(fields: Vec<Type>) -> LuaResult<Self> {
        let mut offsets = Vec::<usize>::with_capacity(fields.len());
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
        let size = unsafe { (*struct_type.as_raw_ptr()).size };

        Ok(Self {
            // libffi_cif: libffi_cfi,
            // fields,
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
        let fields = libffi_type_list_from_table(lua, &table)?;
        let cstruct = lua.create_userdata(Self::new(fields)?)?;
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

    // pub fn get_fields(&self) -> &Vec<Type> {
    //     &self.fields
    // }

    pub fn get_type(&self) -> &Type {
        &self.struct_type
    }
}

impl NativeSized for CStruct {
    fn get_size(&self) -> usize {
        self.size
    }
}

impl NativeConvert for CStruct {
    // Convert luavalue into data, then write into ptr
    unsafe fn luavalue_into<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: Ref<dyn NativeDataHandle>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()> {
        let inner = get_association(lua, CSTRUCT_INNER, this)?
            .ok_or(LuaError::external("Field table not found"))?;
        let LuaValue::Table(fields) = inner else {
            return Err(LuaError::external("Field table not found"));
        };
        let LuaValue::Table(ref table) = value else {
            return Err(LuaError::external("Value is not a table"));
        };

        for i in 0..fields.raw_len() {
            let offset = self.offset(i)?;
            let LuaValue::UserData(ref ctype) = fields.raw_get(i + 1)? else {
                return Err(LuaError::external("Failed to read field table"));
            };

            let data: LuaValue = table.get(i + 1)?;
            if let LuaValue::UserData(userdata) = data {
                let size: usize = type_size_from_userdata(ctype)?;

                // Copy data from Box
                if userdata.is::<FfiBox>() {
                    let mut ffibox: std::cell::RefMut<'_, FfiBox> =
                        userdata.borrow_mut::<FfiBox>()?;
                    if !ffibox.check_boundary(offset + size) {
                        return Err(LuaError::external(format!(
                            "Out of bounds. Read data from {} argument failed",
                            i + 1
                        )));
                    }
                    unsafe {
                        ptr.cast::<u8>()
                            .copy_from(ffibox.get_ptr().byte_offset(offset as isize), size);
                    }

                // Copy data from ref
                } else if userdata.is::<FfiRef>() {
                    let ffiref = userdata.borrow::<FfiRef>()?;
                    if !ffiref.boundary.check_sized(0, size) {
                        return Err(LuaError::external(format!(
                            "Out of bounds. Read data from {} argument failed",
                            i + 1
                        )));
                    }
                    unsafe {
                        ptr.cast::<u8>().copy_from(
                            ffiref.get_ptr().byte_offset(offset as isize).cast::<u8>(),
                            size,
                        );
                    }
                }
            } else {
                luavalue_into_ptr(ctype, lua, data, unsafe {
                    ptr.byte_offset(offset as isize)
                })?;
            }
        }

        Ok(())
    }

    // Read data from ptr, then convert into luavalue
    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: Ref<dyn NativeDataHandle>,
    ) -> LuaResult<LuaValue<'lua>> {
        let inner = get_association(lua, CSTRUCT_INNER, this)?
            .ok_or(LuaError::external("Field table not found"))?;
        let LuaValue::Table(fields) = inner else {
            return Err(LuaError::external("Field table not found"));
        };
        let len = fields.raw_len();
        let result = lua.create_table_with_capacity(len, 0)?;

        for i in 0..len {
            let offset = self.offset(i)?;
            let LuaValue::UserData(ref ctype) = fields.raw_get(i + 1)? else {
                return Err(LuaError::external("Failed to read field table"));
            };
            let value = ptr_into_luavalue(ctype, lua, unsafe { ptr.byte_offset(offset as isize) })?;
            result.raw_set(i + 1, value)?;
        }

        Ok(LuaValue::Table(result))
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
        methods.add_function("box", |lua, (this, table): (LuaAnyUserData, LuaValue)| {
            let cstruct = this.borrow::<CStruct>()?;
            let mut result = FfiBox::new(cstruct.size);
            cstruct.luavalue_into(&this, lua, table, result.get_ptr().cast())?;
            Ok(result)
        });
        methods.add_function(
            "from",
            |lua, (cstruct, userdata, offset): (LuaAnyUserData, LuaAnyUserData, Option<isize>)| {
                let this = cstruct.borrow::<Self>()?;
                userdata_check_boundary(&userdata, offset.unwrap_or(0), this.size)?
                    .then_some(())
                    .ok_or(LuaError::external("Out of bounds"))?;
                unsafe { this.read_userdata(&cstruct, lua, &userdata, offset) }
            },
        );
        methods.add_function(
            "into",
            |lua,
             (cstruct, userdata, value, offset): (
                LuaAnyUserData,
                LuaAnyUserData,
                LuaValue,
                Option<isize>,
            )| {
                let this = cstruct.borrow::<Self>()?;
                userdata_check_boundary(&userdata, offset.unwrap_or(0), this.size)?
                    .then_some(())
                    .ok_or(LuaError::external("Out of bounds"))?;
                unsafe { this.write_userdata(&cstruct, lua, value, userdata, offset) }
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
