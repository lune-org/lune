#![allow(clippy::cargo_common_metadata)]

use libffi::middle::Type;
use mlua::prelude::*;

use super::association_names::CPTR_INNER;
use super::c_arr::CArr;
use super::c_helper::{name_from_userdata, stringify_userdata};
use crate::ffi::ffi_association::{get_association, set_association};

pub struct CPtr();

impl CPtr {
    // Create pointer type with '.inner' field
    // inner can be CArr, CType or CStruct
    pub fn from_lua_userdata<'lua>(
        lua: &'lua Lua,
        inner: &LuaAnyUserData,
    ) -> LuaResult<LuaValue<'lua>> {
        let value = Self().into_lua(lua)?;

        set_association(lua, CPTR_INNER, &value, inner)?;

        Ok(value)
    }

    // Stringify CPtr with inner ctype
    pub fn stringify(userdata: &LuaAnyUserData) -> LuaResult<String> {
        let inner: LuaValue = userdata.get("inner")?;

        if inner.is_userdata() {
            let inner = inner
                .as_userdata()
                .ok_or(LuaError::external("failed to get inner type userdata."))?;
            Ok(format!(
                " <{}({})> ",
                name_from_userdata(inner),
                stringify_userdata(inner)?,
            ))
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }

    // Return void*
    pub fn get_type() -> Type {
        Type::pointer()
    }
}

impl LuaUserData for CPtr {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, _| Ok(size_of::<usize>()));
        fields.add_field_function_get("inner", |lua, this| {
            let inner = get_association(lua, CPTR_INNER, this)?
                .ok_or(LuaError::external("inner type not found"))?;
            Ok(inner)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            let pointer = CPtr::from_lua_userdata(lua, &this)?;
            Ok(pointer)
        });
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            let carr = CArr::from_lua_userdata(lua, &this, length)?;
            Ok(carr)
        });
        methods.add_meta_function(LuaMetaMethod::ToString, |_, this: LuaAnyUserData| {
            let name: Result<String, LuaError> = CPtr::stringify(&this);
            Ok(name)
        });
    }
}
