use crate::ffi::bit_field::*;

pub enum BoxFlag {
    Leaked,
}

impl BoxFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK1,
        }
    }
}
