use std::cell::Ref;

use libffi::middle::Type;
use mlua::prelude::*;

use super::{
    association_names::CARR_INNER,
    c_helper::{get_conv, libffi_type_from_userdata, pretty_format_userdata},
    CPtr,
};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    FfiBox, GetNativeData, NativeConvert, NativeData, NativeSize,
};
use crate::libffi_helper::get_ensured_size;

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays.

// See: https://stackoverflow.com/a/43525176

// Padding after each field inside the struct is set to next field can follow the alignment.
// There is no problem even if you create a struct with n fields of a single type within the struct. Array adheres to the condition that there is no additional padding between each element. Padding to a struct is padding inside the struct. Simply think of the padding byte as a trailing unnamed field.

pub struct CArr {
    // element_type: Type,
    struct_type: Type,
    length: usize,
    field_size: usize,
    size: usize,
    conv: *const dyn NativeConvert,
}

impl CArr {
    pub fn new(
        element_type: Type,
        length: usize,
        conv: *const dyn NativeConvert,
    ) -> LuaResult<Self> {
        let field_size = get_ensured_size(element_type.as_raw_ptr())?;
        let struct_type = Type::structure(vec![element_type.clone(); length]);

        Ok(Self {
            // element_type,
            struct_type,
            length,
            field_size,
            size: field_size * length,
            conv,
        })
    }

    pub fn new_from_lua_userdata<'lua>(
        lua: &'lua Lua,
        luatype: &LuaAnyUserData<'lua>,
        length: usize,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = libffi_type_from_userdata(lua, luatype)?;
        let conv = unsafe { get_conv(luatype)? };
        let carr = lua.create_userdata(Self::new(fields, length, conv)?)?;

        set_association(lua, CARR_INNER, &carr, luatype)?;
        Ok(carr)
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_type(&self) -> &Type {
        &self.struct_type
    }

    // pub fn get_element_type(&self) -> &Type {
    //     &self.element_type
    // }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let inner: LuaValue = userdata.get("inner")?;
        let carr = userdata.borrow::<CArr>()?;

        if inner.is_userdata() {
            let inner = inner
                .as_userdata()
                .ok_or(LuaError::external("failed to get inner type userdata."))?;

            Ok(format!(
                "{}*{}",
                pretty_format_userdata(lua, inner)?,
                carr.length,
            ))
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }
}

impl NativeSize for CArr {
    fn get_size(&self) -> usize {
        self.size
    }
}
impl NativeConvert for CArr {
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
        for i in 0..self.length {
            let field_offset = (i * self.field_size) as isize;
            let data: LuaValue = table.get(i + 1)?;

            self.conv.as_ref().unwrap().luavalue_into(
                lua,
                field_offset + offset,
                data_handle,
                data,
            )?;
        }
        Ok(())
    }

    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn NativeData>,
    ) -> LuaResult<LuaValue<'lua>> {
        let table = lua.create_table_with_capacity(self.length, 0)?;
        for i in 0..self.length {
            let field_offset = (i * self.field_size) as isize;
            table.set(
                i + 1,
                self.conv.as_ref().unwrap().luavalue_from(
                    lua,
                    field_offset + offset,
                    data_handle,
                )?,
            )?;
        }
        Ok(LuaValue::Table(table))
    }
}

impl LuaUserData for CArr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.get_size()));
        fields.add_field_method_get("length", |_, this| Ok(this.get_length()));
        fields.add_field_function_get("inner", |lua, this: LuaAnyUserData| {
            let inner: LuaValue = get_association(lua, CARR_INNER, this)?
                // It shouldn't happen.
                .ok_or(LuaError::external("inner field not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("offset", |_, this, offset: isize| {
            if this.length > (offset as usize) && offset >= 0 {
                Ok(this.field_size * (offset as usize))
            } else {
                Err(LuaError::external("Out of index"))
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

                unsafe { this.luavalue_from(lua, offset, data_handle) }
            },
        );
        methods.add_method(
            "into",
            |lua, this, (userdata, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.size) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { this.luavalue_into(lua, offset, data_handle, value) }
            },
        );
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::new_from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |lua, this: LuaAnyUserData| {
            let result = CArr::stringify(lua, &this)?;
            Ok(result)
        });
    }
}
