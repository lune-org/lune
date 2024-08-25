use std::ffi::c_void;

use mlua::prelude::*;

use super::ffi_box::FfiBox;
use super::ffi_ref::FfiRef;

// Converts ffi status into &str
pub const FFI_STATUS_NAMES: [&str; 4] = [
    "ffi_status_FFI_OK",
    "ffi_status_FFI_BAD_TYPEDEF",
    "ffi_status_FFI_BAD_ABI",
    "ffi_status_FFI_BAD_ARGTYPE",
];

pub unsafe fn get_ptr_from_userdata(
    userdata: &LuaAnyUserData,
    offset: Option<isize>,
) -> LuaResult<*mut c_void> {
    let ptr = if userdata.is::<FfiBox>() {
        userdata.borrow::<FfiBox>()?.get_ptr()
    } else if userdata.is::<FfiRef>() {
        userdata.borrow::<FfiRef>()?.get_ptr()
    } else {
        return Err(LuaError::external("asdf"));
    };

    let ptr = if let Some(t) = offset {
        ptr.offset(t)
    } else {
        ptr
    };

    Ok(ptr)
}
