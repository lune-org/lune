use core::ffi::c_char;
use std::vec::Vec;

pub const CHAR_IS_SIGNED: bool = c_char::MIN as u8 != u8::MIN;

pub fn get_platform_value() -> Vec<(&'static str, &'static str)> {
    vec![(
        "char_variant",
        if CHAR_IS_SIGNED { "schar" } else { "uchar" },
    )]
}
