use libffi::middle::Type;
use mlua::prelude::*;

use crate::ffi::{FfiSignedness, FfiSize};

use super::method_provider;

pub struct CVoidInfo();

impl FfiSignedness for CVoidInfo {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl FfiSize for CVoidInfo {
    fn get_size(&self) -> usize {
        0
    }
}
impl CVoidInfo {
    pub fn new() -> Self {
        Self()
    }
    pub fn get_middle_type() -> Type {
        Type::void()
    }
}

impl LuaUserData for CVoidInfo {}
