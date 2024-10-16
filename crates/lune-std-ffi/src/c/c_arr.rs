use std::cell::Ref;

use libffi::middle::Type;
use mlua::prelude::*;

use super::{association_names::CARR_INNER, c_helper, method_provider};
use crate::ffi::{
    ffi_association::{get_association, set_association},
    NativeConvert, NativeData, NativeSize,
};
use crate::libffi_helper::get_ensured_size;

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays. (flatten)

// We can simply provide array type with struct.
// See: https://stackoverflow.com/a/43525176

pub struct CArr {
    struct_type: Type,
    length: usize,
    size: usize,
    inner_size: usize,
    inner_conv: *const dyn NativeConvert,
}

impl CArr {
    pub fn new(
        element_type: Type,
        length: usize,
        inner_conv: *const dyn NativeConvert,
    ) -> LuaResult<Self> {
        let inner_size = get_ensured_size(element_type.as_raw_ptr())?;
        let struct_type = Type::structure(vec![element_type.clone(); length]);

        Ok(Self {
            // element_type,
            struct_type,
            length,
            size: inner_size * length,
            inner_size,
            inner_conv,
        })
    }

    pub fn from_userdata<'lua>(
        lua: &'lua Lua,
        type_userdata: &LuaAnyUserData<'lua>,
        length: usize,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = c_helper::get_middle_type(type_userdata)?;
        let conv = unsafe { c_helper::get_conv(type_userdata)? };
        let carr = lua.create_userdata(Self::new(fields, length, conv)?)?;

        set_association(lua, CARR_INNER, &carr, type_userdata)?;
        Ok(carr)
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_type(&self) -> Type {
        self.struct_type.clone()
    }

    // Stringify for pretty printing like:
    // <CArr( u8, length = 8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let this = userdata.borrow::<CArr>()?;
        if let Some(LuaValue::UserData(inner_userdata)) =
            get_association(lua, CARR_INNER, userdata)?
        {
            Ok(format!(
                " {}, length = {} ",
                c_helper::pretty_format(lua, &inner_userdata)?,
                this.length,
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
            let field_offset = (i * self.inner_size) as isize;
            let data: LuaValue = table.get(i + 1)?;

            self.inner_conv.as_ref().unwrap().luavalue_into(
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
            let field_offset = (i * self.inner_size) as isize;
            table.set(
                i + 1,
                self.inner_conv.as_ref().unwrap().luavalue_from(
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
        // Subtype
        method_provider::provide_ptr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        method_provider::provide_box(methods);
        method_provider::provide_from(methods);
        method_provider::provide_into(methods);

        methods.add_method("offset", |_, this, offset: isize| {
            if this.length > (offset as usize) && offset >= 0 {
                Ok(this.inner_size * (offset as usize))
            } else {
                Err(LuaError::external("Out of index"))
            }
        });
    }
}
