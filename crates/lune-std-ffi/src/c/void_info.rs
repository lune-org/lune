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
    pub fn stringify() -> LuaResult<String> {
        Ok(String::from("CVoid"))
    }
}

impl LuaUserData for CVoidInfo {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, _| Ok(0));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        method_provider::provide_to_string(methods);
        method_provider::provide_ptr(methods);
    }
}
