use std::ptr::{self, null_mut};

use libffi::{low, raw};
use mlua::prelude::*;

pub const SIZE_OF_POINTER: usize = size_of::<*mut ()>();

// libffi widens any integral return value narrower than `ffi_arg` to a full
// `ffi_arg` word when writing it through the result pointer. Callers must
// therefore provide a result buffer of at least this many bytes.
// See: <http://www.chiark.greenend.org.uk/doc/libffi-dev/html/The-Basics.html>
//
// libffi-sys binds `ffi_arg` as c_ulong, which is 4 bytes on LLP64 targets
// (windows) even though the C definition is pointer sized there, so clamp
// to at least the size of a pointer
pub const SIZE_OF_FFI_ARG: usize = {
    if size_of::<raw::ffi_arg>() > size_of::<usize>() {
        size_of::<raw::ffi_arg>()
    } else {
        size_of::<usize>()
    }
};

// Get ensured size of ctype (raw::libffi_type)
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

// Converts ffi status into &str for formatting
const FFI_STATUS_NAMES: [&str; 4] = [
    "ffi_status_FFI_OK",
    "ffi_status_FFI_BAD_TYPEDEF",
    "ffi_status_FFI_BAD_ABI",
    "ffi_status_FFI_BAD_ARGTYPE",
];

// Check ffi_result is OK
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
