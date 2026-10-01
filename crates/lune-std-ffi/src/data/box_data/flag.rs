use crate::ffi::bit_field::*;

pub enum BoxFlag {
    Leaked,
    Freed,
}

impl BoxFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK1,
            Self::Freed => U8_MASK2,
        }
    }
}
