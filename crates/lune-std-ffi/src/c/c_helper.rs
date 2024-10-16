#![allow(clippy::inline_always)]

use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{
    association_names::CTYPE_STATIC,
    types::{get_ctype_conv, get_ctype_size},
    CArr, CPtr, CStruct, CTypeStatic,
};
use crate::ffi::{ffi_association::get_association, NativeConvert, NativeSize};

pub fn get_userdata(value: LuaValue) -> LuaResult<LuaAnyUserData> {
    if let LuaValue::UserData(field_type) = value {
        Ok(field_type)
    } else {
        Err(LuaError::external(format!(
            "Unexpected field. CStruct, CType or CArr is required for element but got {}",
            pretty_format_value(&value, &ValueFormatConfig::new())
        )))
    }
}

// Get the NativeConvert handle from the type UserData
// this is intended to avoid lookup userdata and lua table every time. (eg: struct)
// userdata must live longer than the NativeConvert handle.
// However, c_struct is a strong reference to each field, so this is not a problem.
pub unsafe fn get_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn NativeConvert> {
    if userdata.is::<CStruct>() {
        Ok(userdata.to_pointer().cast::<CStruct>() as *const dyn NativeConvert)
    } else {
        get_ctype_conv(userdata)
    }
}

pub unsafe fn get_conv_list_from_table(
    table: &LuaTable,
) -> LuaResult<Vec<*const dyn NativeConvert>> {
    let len: usize = table.raw_len();
    let mut conv_list = Vec::<*const dyn NativeConvert>::with_capacity(len);

    for i in 0..len {
        let value: LuaValue = table.raw_get(i + 1)?;
        conv_list.push(get_conv(&get_userdata(value)?)?);
    }

    Ok(conv_list)
}

pub fn get_size(this: &LuaAnyUserData) -> LuaResult<usize> {
    if this.is::<CStruct>() {
        Ok(this.borrow::<CStruct>()?.get_size())
    } else if this.is::<CArr>() {
        Ok(this.borrow::<CArr>()?.get_size())
    } else {
        get_ctype_size(this)
    }
}

// get Vec<libffi_type> from table(array) of c-type userdata
pub fn libffi_type_list_from_table(lua: &Lua, table: &LuaTable) -> LuaResult<Vec<Type>> {
    let len: usize = table.raw_len();
    let mut fields = Vec::with_capacity(len);

    for i in 0..len {
        // Test required
        let value = table.raw_get(i + 1)?;
        if let LuaValue::UserData(field_type) = value {
            fields.push(libffi_type_from_userdata(lua, &field_type)?);
        } else {
            return Err(LuaError::external(format!(
                "Unexpected field. CStruct, CType or CArr is required for element but got {}",
                value.type_name()
            )));
        }
    }

    Ok(fields)
}

// get libffi_type from any c-type userdata
pub fn libffi_type_from_userdata(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStruct>() {
        Ok(userdata.borrow::<CStruct>()?.get_type().to_owned())
    } else if let Some(t) = get_association(lua, CTYPE_STATIC, userdata)? {
        Ok(t.as_userdata()
            .ok_or_else(|| LuaError::external("Failed to get static ctype from userdata"))?
            .borrow::<CTypeStatic>()?
            .libffi_type
            .clone())
    } else if userdata.is::<CArr>() {
        Ok(userdata.borrow::<CArr>()?.get_type().to_owned())
    } else if userdata.is::<CPtr>() {
        Ok(CPtr::get_type())
    } else {
        Err(LuaError::external(format!(
            "Unexpected field. CStruct, CType, CString or CArr is required for element but got {}",
            pretty_format_value(
                // Since the data is in the Lua location,
                // there is no problem with the clone.
                &LuaValue::UserData(userdata.to_owned()),
                &ValueFormatConfig::new()
            )
        )))
    }
}

// stringify any c-type userdata (for recursive)
pub fn stringify_userdata(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    if userdata.is::<CStruct>() {
        let name = CStruct::stringify(lua, userdata)?;
        Ok(name)
    } else if userdata.is::<CArr>() {
        let name = CArr::stringify(lua, userdata)?;
        Ok(name)
    } else if userdata.is::<CPtr>() {
        let name: String = CPtr::stringify(lua, userdata)?;
        Ok(name)
    // Get CTypeStatic from CType<Any>
    } else if let Some(t) = get_association(lua, CTYPE_STATIC, userdata)? {
        Ok(String::from(
            t.as_userdata()
                .ok_or_else(|| LuaError::external("Failed to get static ctype from userdata"))?
                .borrow::<CTypeStatic>()?
                .name
                .unwrap_or("unnamed"),
        ))
    } else {
        Ok(String::from("unnamed"))
    }
}

// get name tag for any c-type userdata
pub fn tagname_from_userdata(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    Ok(if userdata.is::<CStruct>() {
        String::from("CStruct")
    } else if userdata.is::<CArr>() {
        String::from("CArr")
    } else if userdata.is::<CPtr>() {
        String::from("CPtr")
    } else if userdata_is_ctype(lua, userdata)? {
        String::from("CType")
    } else {
        String::from("unnamed")
    })
}

pub fn userdata_is_ctype(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<bool> {
    Ok(get_association(lua, CTYPE_STATIC, userdata)?.is_some())
}

// emulate 'print' for ctype userdata, but ctype is simplified
pub fn pretty_format_userdata(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    if userdata_is_ctype(lua, userdata)? {
        stringify_userdata(lua, userdata)
    } else {
        Ok(format!(
            "<{}({})>",
            tagname_from_userdata(lua, userdata)?,
            stringify_userdata(lua, userdata)?
        ))
    }
}
