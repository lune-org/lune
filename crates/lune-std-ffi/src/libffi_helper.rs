use std::ptr::{self, null_mut};

use libffi::{low, raw};
use mlua::prelude::*;

use crate::ffi::FFI_STATUS_NAMES;

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

    if result != raw::ffi_status_FFI_OK {
        return Err(LuaError::external(format!(
            "ffi_prep_cif failed. expected result {}, got {}",
            FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[result as usize]
        )));
    }
    unsafe { Ok((*ffi_type).size) }
}

pub const SIEE_OF_POINTER: usize = size_of::<*mut ()>();
