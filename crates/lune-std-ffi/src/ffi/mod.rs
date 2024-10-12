pub mod ffi_association;
mod ffi_box;
mod ffi_callable;
mod ffi_closure;
mod ffi_lib;
mod ffi_native;
mod ffi_raw;
mod ffi_ref;

use mlua::prelude::*;

pub use self::{
    ffi_box::FfiBox,
    ffi_callable::FfiCallable,
    ffi_closure::FfiClosure,
    ffi_lib::FfiLib,
    ffi_native::{
        native_num_cast, FfiArgRefOption, GetNativeData, NativeArgInfo, NativeArgType,
        NativeConvert, NativeData, NativeResultInfo, NativeResultType, NativeSignedness,
        NativeSize,
    },
    ffi_raw::FfiRaw,
    ffi_ref::{create_nullptr, FfiRef, FfiRefFlag},
};

// Named registry table names
mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
    pub const SYM_INNER: &str = "__syn_inner";
}

// Converts ffi status into &str
pub const FFI_STATUS_NAMES: [&str; 4] = [
    "ffi_status_FFI_OK",
    "ffi_status_FFI_BAD_TYPEDEF",
    "ffi_status_FFI_BAD_ABI",
    "ffi_status_FFI_BAD_ARGTYPE",
];

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

    #[inline]
    pub fn u8_test(bits: u8, mask: u8) -> bool {
        bits & mask != 0
    }

    #[inline]
    pub fn u8_test_not(bits: u8, mask: u8) -> bool {
        bits & mask == 0
    }

    #[inline]
    pub fn u8_set(bits: u8, mask: u8, val: bool) -> u8 {
        if val {
            bits | mask
        } else {
            bits & !mask
        }
    }
}

#[inline]
pub fn is_integer(num: LuaValue) -> bool {
    num.is_integer()
}
