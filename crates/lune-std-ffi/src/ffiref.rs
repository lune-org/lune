use core::ffi::c_void;
use mlua::prelude::*;

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRef(*mut c_void);

impl FfiRef {
    pub fn new(target: *mut c_void) -> Self {
        Self(target)
    }
}

impl LuaUserData for FfiRef {}
