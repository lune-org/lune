use core::ffi::c_void;
use mlua::prelude::*;

pub struct LuaRef(*mut c_void);

impl LuaRef {
    pub fn new(target: *mut c_void) -> Self {
        Self(target)
    }
}

impl LuaUserData for LuaRef {}
