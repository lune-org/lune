use crate::ffi::bit_mask::*;

pub enum BoxFlag {
    Leaked,
}

impl BoxFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK2,
        }
    }
}
