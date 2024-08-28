use super::super::bit_mask::*;

pub enum FfiRefFlag {
    Dereferenceable,
    Readable,
    Writable,
    Offsetable,
    Function,
}
impl FfiRefFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Dereferenceable => U8_MASK1,
            Self::Readable => U8_MASK2,
            Self::Writable => U8_MASK3,
            Self::Offsetable => U8_MASK4,
            Self::Function => U8_MASK5,
        }
    }
}

pub struct FfiRefFlagList(u8);
#[allow(unused)]
impl FfiRefFlagList {
    pub fn zero() -> Self {
        Self(0)
    }
    pub fn new(flags: &[FfiRefFlag]) -> Self {
        let mut value = 0;
        for i in flags {
            value |= i.value();
        }
        Self(value)
    }
    fn set(&mut self, value: bool, mask: u8) {
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
    pub fn is_dereferenceable(&self) -> bool {
        U8_TEST!(self.0, U8_MASK1)
    }
    pub fn set_dereferenceable(&mut self, value: bool) {
        self.set(value, U8_MASK1);
    }
    pub fn is_readable(&self) -> bool {
        U8_TEST!(self.0, U8_MASK2)
    }
    pub fn set_readable(&mut self, value: bool) {
        self.set(value, U8_MASK2);
    }
    pub fn is_writable(&self) -> bool {
        U8_TEST!(self.0, U8_MASK3)
    }
    pub fn set_writable(&mut self, value: bool) {
        self.set(value, U8_MASK2);
    }
    pub fn is_offsetable(&self) -> bool {
        U8_TEST!(self.0, U8_MASK4)
    }
    pub fn set_offsetable(&mut self, value: bool) {
        self.set(value, U8_MASK2);
    }
}
impl Clone for FfiRefFlagList {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
