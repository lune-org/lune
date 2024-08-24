use std::ptr::{self, null_mut};

use libffi::{low, middle::Type, raw};
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::c_arr::CArr;
use super::c_ptr::CPtr;
use super::c_struct::CStruct;
use super::c_type::CType;
use crate::ffi::ffi_helper::FFI_STATUS_NAMES;

// get Vec<libffi_type> from table(array) of c-types userdata
pub fn type_list_from_table(table: &LuaTable) -> LuaResult<Vec<Type>> {
    let len: usize = table.raw_len();
    let mut fields = Vec::with_capacity(len);

    for i in 0..len {
        // Test required
        let value = table.raw_get(i + 1)?;
        match value {
            LuaValue::UserData(field_type) => {
                fields.push(type_from_userdata(&field_type)?);
            }
            _ => {
                return Err(LuaError::external(format!(
                    "Unexpected field. CStruct, CType or CArr is required for element but got {}",
                    pretty_format_value(&value, &ValueFormatConfig::new())
                )));
            }
        }
    }

    Ok(fields)
}

// get libffi_type from any c-type userdata
pub fn type_from_userdata(userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStruct>() {
        Ok(userdata.borrow::<CStruct>()?.get_type())
    } else if userdata.is::<CType>() {
        Ok(userdata.borrow::<CType>()?.get_type())
    } else if userdata.is::<CArr>() {
        Ok(userdata.borrow::<CArr>()?.get_type())
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
pub fn stringify_userdata(userdata: &LuaAnyUserData) -> LuaResult<String> {
    if userdata.is::<CType>() {
        let name = userdata.borrow::<CType>()?.stringify();
        Ok(name)
    } else if userdata.is::<CStruct>() {
        let name = CStruct::stringify(userdata)?;
        Ok(name)
    } else if userdata.is::<CArr>() {
        let name = CArr::stringify(userdata)?;
        Ok(name)
    } else if userdata.is::<CPtr>() {
        let name: String = CPtr::stringify(userdata)?;
        Ok(name)
    } else {
        Ok(String::from("unnamed"))
    }
}

// get name tag for any c-type userdata
pub fn name_from_userdata(userdata: &LuaAnyUserData) -> String {
    if userdata.is::<CStruct>() {
        String::from("CStruct")
    } else if userdata.is::<CType>() {
        String::from("CType")
    } else if userdata.is::<CArr>() {
        String::from("CArr")
    } else if userdata.is::<CPtr>() {
        String::from("CPtr")
    } else {
        String::from("unnamed")
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
