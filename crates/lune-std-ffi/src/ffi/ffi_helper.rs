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

// TODO: using trait
// Get raw pointer from userdata
// TODO: boundary check
pub unsafe fn get_ptr_from_userdata(
    userdata: &LuaAnyUserData,
    offset: Option<isize>,
) -> LuaResult<*mut ()> {
    let ptr = if userdata.is::<FfiBox>() {
        userdata.borrow_mut::<FfiBox>()?.get_ptr().cast()
    } else if userdata.is::<FfiRef>() {
        userdata.borrow::<FfiRef>()?.get_ptr()
    } else {
        return Err(LuaError::external("Unexpected userdata"));
    };

    let ptr = if let Some(t) = offset {
        ptr.cast::<u8>().offset(t).cast()
    } else {
        ptr
    };

    Ok(ptr)
}
