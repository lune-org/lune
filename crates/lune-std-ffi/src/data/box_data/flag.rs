use crate::ffi::bit_mask::*;

pub enum BoxDataFlag {
    Leaked,
}

impl BoxDataFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK2,
        }
    }
}
