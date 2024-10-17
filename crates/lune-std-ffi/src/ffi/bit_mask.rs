#![allow(unused)]

pub const U8_MASK1: u8 = 1;
pub const U8_MASK2: u8 = 2;
pub const U8_MASK3: u8 = 4;
pub const U8_MASK4: u8 = 8;
pub const U8_MASK5: u8 = 16;
pub const U8_MASK6: u8 = 32;
pub const U8_MASK7: u8 = 64;
pub const U8_MASK8: u8 = 128;

#[inline]
pub fn u8_test(bits: u8, mask: u8) -> bool {
    bits & mask != 0
}

#[inline]
pub fn u8_test_not(bits: u8, mask: u8) -> bool {
    bits & mask == 0
}

#[inline]
pub fn u8_set(bits: u8, mask: u8, val: bool) -> u8 {
    if val {
        bits | mask
    } else {
        bits & !mask
    }
}
