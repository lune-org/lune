use libffi::middle::Type;
use mlua::prelude::*;

use super::association_names::CARR_INNER;
use super::c_helper::{
    get_ensured_size, name_from_userdata, stringify_userdata, type_from_userdata,
};
use super::c_ptr::CPtr;
use super::c_type::CType;
use crate::ffi::ffi_association::{get_association, set_association};

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays.

// See: https://stackoverflow.com/a/43525176

// Padding after each field inside the struct is set to next field can follow the alignment.
// There is no problem even if you create a struct with n fields of a single type within the struct. Array adheres to the condition that there is no additional padding between each element. Padding to a struct is padding inside the struct. Simply think of the padding byte as a trailing unnamed field.

pub struct CArr {
    element_type: Type,
    struct_type: Type,
    length: usize,
    field_size: usize,
    size: usize,
}

impl CArr {
    pub fn new(element_type: Type, length: usize) -> LuaResult<Self> {
        let struct_type = Type::structure(vec![element_type.clone(); length]);
        let field_size = get_ensured_size(element_type.as_raw_ptr())?;

        Ok(Self {
            element_type,
            struct_type,
            length,
            field_size,
            size: field_size * length,
        })
    }

    pub fn from_lua_userdata<'lua>(
        lua: &'lua Lua,
        luatype: &LuaAnyUserData<'lua>,
        length: usize,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let fields = type_from_userdata(luatype)?;
        let carr = lua.create_userdata(Self::new(fields, length)?)?;

        set_association(lua, CARR_INNER, &carr, luatype)?;
        Ok(carr)
    }

    pub fn get_type(&self) -> Type {
        self.struct_type.clone()
    }

    // pub fn get_element_type(&self) -> Type {
    //     self.element_type.clone()
    // }

    // Stringify cstruct for pretty printing something like:
    // <CStruct( u8, i32, size = 8 )>
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let inner: LuaValue = userdata.get("inner")?;
        let carr = userdata.borrow::<CArr>()?;

        if inner.is_userdata() {
            let inner = inner
                .as_userdata()
                .ok_or(LuaError::external("failed to get inner type userdata."))?;

            if inner.is::<CType>() {
                Ok(format!(
                    " {} ; {} ",
                    stringify_userdata(inner)?,
                    carr.length
                ))
            } else {
                Ok(format!(
                    " <{}({})> ; {} ",
                    name_from_userdata(inner),
                    stringify_userdata(inner)?,
                    carr.length
                ))
            }
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }
}

impl LuaUserData for CArr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size));
        fields.add_field_method_get("length", |_, this| Ok(this.length));
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
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let result = CArr::stringify(&this)?;
            Ok(result)
        });
    }
}
