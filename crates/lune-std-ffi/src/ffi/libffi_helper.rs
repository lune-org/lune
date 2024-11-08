use std::ptr::{self, null_mut};

use libffi::{low, raw};
use mlua::prelude::*;

// Get ensured size of c-type (raw::libffi_type)
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

    ffi_status_assert(result)?;
    unsafe { Ok((*ffi_type).size) }
}

pub const SIZE_OF_POINTER: usize = size_of::<*mut ()>();

// Converts ffi status into &str
pub const FFI_STATUS_NAMES: [&str; 4] = [
    "ffi_status_FFI_OK",
    "ffi_status_FFI_BAD_TYPEDEF",
    "ffi_status_FFI_BAD_ABI",
    "ffi_status_FFI_BAD_ARGTYPE",
];

pub fn ffi_status_assert(result: raw::ffi_status) -> LuaResult<()> {
    if result == raw::ffi_status_FFI_OK {
        Ok(())
    } else {
        Err(LuaError::external(format!(
            "ffi_status assertion failed. expected result {}, got {}",
            FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[result as usize]
        )))
    }
}
