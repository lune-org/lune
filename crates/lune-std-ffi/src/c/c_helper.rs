#![allow(clippy::inline_always)]

use std::ptr::{self, null_mut};

use libffi::{low, middle::Type, raw};
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{
    association_names::CTYPE_STATIC, types::get_ctype_conv, CArr, CPtr, CStruct, CTypeStatic,
};
use crate::ffi::{ffi_association::get_association, NativeConvert, FFI_STATUS_NAMES};

// Get the NativeConvert handle from the type UserData
// this is intended to avoid lookup userdata and lua table every time. (eg: struct)
// userdata must live longer than the NativeConvert handle.
// However, c_struct is a strong reference to each field, so this is not a problem.
pub unsafe fn get_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn NativeConvert> {
    if userdata.is::<CStruct>() {
        Ok(userdata.to_pointer().cast::<CStruct>() as *const dyn NativeConvert)
    } else {
        unsafe { get_ctype_conv(userdata) }
    }
}
pub unsafe fn get_conv_list_from_table(
    table: &LuaTable,
) -> LuaResult<Vec<*const dyn NativeConvert>> {
    let len: usize = table.raw_len();
    let mut conv_list = Vec::<*const dyn NativeConvert>::with_capacity(len);

    for i in 0..len {
        let value: LuaValue = table.raw_get(i + 1)?;

        if let LuaValue::UserData(field_type) = value {
            conv_list.push(get_conv(&field_type)?);
        } else {
            return Err(LuaError::external(format!(
                "Unexpected field. CStruct, CType or CArr is required for element but got {}",
                pretty_format_value(&value, &ValueFormatConfig::new())
            )));
        }
    }

    Ok(conv_list)
}

// #[inline(always)]
// pub fn type_size_from_userdata(this: &LuaAnyUserData) -> LuaResult<usize> {
//     if this.is::<CStruct>() {
//         Ok(this.borrow::<CStruct>()?.get_size())
//     } else if this.is::<CArr>() {
//         Ok(this.borrow::<CArr>()?.get_size())
//     } else {
//         ctype_size_from_userdata(this)
//     }
// }

// get Vec<libffi_type> from table(array) of c-types userdata
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
            .ok_or(LuaError::external(
                "Failed to get static ctype from userdata",
            ))?
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
                .ok_or(LuaError::external(
                    "Failed to get static ctype from userdata",
                ))?
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

// Ensure sizeof c-type (raw::libffi_type)
// See: http://www.chiark.greenend.org.uk/doc/libffi-dev/html/Size-and-Alignment.html
pub fn get_ensured_size(ffi_type: *mut raw::ffi_type) -> LuaResult<usize> {
    let mut cif = low::ffi_cif::default();
    let result = unsafe {
        raw::ffi_prep_cif(
            ptr::from_mut(&mut cif),
            raw::ffi_abi_FFI_DEFAULT_ABI,
            0,
            ffi_type,
            null_mut(),
        )
    };

    if result != raw::ffi_status_FFI_OK {
        return Err(LuaError::external(format!(
            "ffi_get_struct_offsets failed. expected result {}, got {}",
            FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[result as usize]
        )));
    }
    unsafe { Ok((*ffi_type).size) }
}
