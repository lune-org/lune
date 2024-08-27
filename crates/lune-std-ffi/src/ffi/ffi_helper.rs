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

#[allow(unused)]
pub mod bit_mask {
    pub const U8_MASK1: u8 = 1;
    pub const U8_MASK2: u8 = 2;
    pub const U8_MASK3: u8 = 4;
    pub const U8_MASK4: u8 = 8;
    pub const U8_MASK5: u8 = 16;
    pub const U8_MASK6: u8 = 32;
    pub const U8_MASK7: u8 = 64;
    pub const U8_MASK8: u8 = 128;

    macro_rules! U8_TEST {
        ($val:expr, $mask:ident) => {
            ($val & $mask != 0)
        };
    }

    pub(crate) use U8_TEST;
}
