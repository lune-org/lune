use core::ffi::c_char;
use std::env::consts;
use std::vec::Vec;

pub const CHAR_IS_SIGNED: bool = c_char::MIN as u8 != u8::MIN;
pub const IS_LITTLE_ENDIAN: bool = cfg!(target_endian = "little");

pub fn get_platform_value() -> Vec<(&'static str, &'static str)> {
    vec![
        // https://doc.rust-lang.org/std/env/consts/constant.ARCH.html
        ("arch", consts::ARCH),
        // https://doc.rust-lang.org/std/env/consts/constant.OS.html
        ("os", consts::OS),
        // https://doc.rust-lang.org/std/env/consts/constant.FAMILY.html
        ("family", consts::FAMILY),
        ("endian", if IS_LITTLE_ENDIAN { "little" } else { "big" }),
        (
            "char_variant",
            if CHAR_IS_SIGNED { "schar" } else { "uchar" },
        ),
    ]
}
