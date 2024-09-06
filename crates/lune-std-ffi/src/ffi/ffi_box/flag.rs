use super::super::bit_mask::*;

pub enum FfiBoxFlag {
    Dropped,
    Leaked,
}

impl FfiBoxFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Dropped => U8_MASK1,
            Self::Leaked => U8_MASK2,
        }
    }
}
