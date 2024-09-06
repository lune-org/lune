use super::super::bit_mask::*;

pub enum FfiRefFlag {
    // Dereferenceable,
    // Readable,
    // Writable,
    // Offsetable,
    // Function,
    // Mutable,
}
impl FfiRefFlag {
    pub const fn value(&self) -> u8 {
        match self {
            Self::Dereferenceable => U8_MASK1,
            Self::Readable => U8_MASK2,
            Self::Writable => U8_MASK3,
            Self::Offsetable => U8_MASK4,
            Self::Function => U8_MASK5,
            Self::Mutable => U8_MASK6,
        }
    }
}

pub struct FfiRefFlagList(u8);
#[allow(unused)]
impl FfiRefFlagList {
    pub const fn zero() -> Self {
        Self(0)
    }
    pub const fn new(flags: u8) -> Self {
        Self(flags)
    }
    pub const fn all() -> Self {
        Self(
            FfiRefFlag::Dereferenceable.value()
                | FfiRefFlag::Readable.value()
                | FfiRefFlag::Writable.value()
                | FfiRefFlag::Offsetable.value()
                | FfiRefFlag::Function.value(),
        )
    }
    fn set(&mut self, value: bool, mask: u8) {
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }
    pub fn is_dereferenceable(&self) -> bool {
        U8_TEST!(self.0, FfiRefFlag::Dereferenceable.value())
    }
    pub fn set_dereferenceable(&mut self, value: bool) {
        self.set(value, FfiRefFlag::Dereferenceable.value());
    }
    pub fn is_readable(&self) -> bool {
        U8_TEST!(self.0, FfiRefFlag::Readable.value())
    }
    pub fn set_readable(&mut self, value: bool) {
        self.set(value, FfiRefFlag::Readable.value());
    }
    pub fn is_writable(&self) -> bool {
        U8_TEST!(self.0, FfiRefFlag::Writable.value())
    }
    pub fn set_writable(&mut self, value: bool) {
        self.set(value, FfiRefFlag::Writable.value());
    }
    pub fn is_offsetable(&self) -> bool {
        U8_TEST!(self.0, FfiRefFlag::Offsetable.value())
    }
    pub fn set_offsetable(&mut self, value: bool) {
        self.set(value, FfiRefFlag::Offsetable.value());
    }
    pub fn is_mutable(&self) -> bool {
        U8_TEST!(self.0, FfiRefFlag::Mutable.value())
    }
    pub fn set_mutable(&mut self, value: bool) {
        self.set(value, FfiRefFlag::Mutable.value());
    }
}
impl Clone for FfiRefFlagList {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
