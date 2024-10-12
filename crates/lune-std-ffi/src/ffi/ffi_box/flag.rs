use super::super::bit_mask::*;

pub enum FfiBoxFlag {
    Leaked,
}

impl FfiBoxFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Leaked => U8_MASK2,
        }
    }
}
