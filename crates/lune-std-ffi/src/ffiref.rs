use core::ffi::c_void;
use std::ptr;

use mlua::prelude::*;

use crate::association::set_association;

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRef(*mut c_void);

const REF_INNER: &str = "__ref_inner";

impl FfiRef {
    pub fn new(target: *mut c_void) -> Self {
        Self(target)
    }

    // bad naming. i have no idea what should i use
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiRef>()?;

        let luaref = lua.create_userdata(FfiRef::new(ptr::from_ref(&target.0) as *mut c_void))?;

        set_association(lua, REF_INNER, luaref.clone(), this.clone())?;

        Ok(luaref)
    }

    pub unsafe fn deref(&self) -> Self {
        Self::new(*self.0.cast::<*mut c_void>())
    }

    pub unsafe fn offset(&self, offset: isize) -> Self {
        Self::new(self.0.offset(offset))
    }
}

impl LuaUserData for FfiRef {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("deref", |_, this, ()| {
            let ffiref = unsafe { this.deref() };
            Ok(ffiref)
        });
        methods.add_method("offset", |_, this, offset: isize| {
            let ffiref = unsafe { this.offset(offset) };
            Ok(ffiref)
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let ffiref = FfiRef::luaref(lua, this)?;
            Ok(ffiref)
        });
    }
}
