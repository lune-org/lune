use std::ptr;

use mlua::prelude::*;

use super::association_names::REF_INNER;
use super::ffi_association::{get_association, set_association};
use super::ffi_bounds::FfiRefBounds;

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRef {
    ptr: *mut (),
    range: Option<FfiRefBounds>,
}

impl FfiRef {
    pub fn new(ptr: *mut (), range: Option<FfiRefBounds>) -> Self {
        Self { ptr, range }
    }

    // Make FfiRef from ref
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiRef>()?;

        let luaref = lua.create_userdata(FfiRef::new(
            ptr::from_ref(&target.ptr) as *mut (),
            Some(FfiRefBounds {
                low: 0,
                high: size_of::<usize>(),
            }),
        ))?;

        // If the ref holds a box, make sure the new ref also holds the box
        if let Some(t) = get_association(lua, REF_INNER, &this)? {
            set_association(lua, REF_INNER, &luaref, t)?;
        }

        Ok(luaref)
    }

    pub fn get_ptr(&self) -> *mut () {
        self.ptr
    }

    pub unsafe fn deref(&self) -> Self {
        Self::new(*self.ptr.cast::<*mut ()>(), None)
    }

    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        if let Some(ref t) = self.range {
            if !t.check(offset) {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. high: {}, low: {}. offset got {}",
                    t.high, t.low, offset
                )));
            }
        }
        let range = self.range.as_ref().map(|t| t.offset(offset));

        Ok(Self::new(
            self.ptr.cast::<u8>().offset(offset).cast(),
            range,
        ))
    }
}

impl LuaUserData for FfiRef {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("deref", |lua, this: LuaAnyUserData| {
            let inner = get_association(lua, REF_INNER, &this)?;
            let ffiref = this.borrow::<FfiRef>()?;
            let result = lua.create_userdata(unsafe { ffiref.deref() })?;

            if let Some(t) = inner {
                set_association(lua, REF_INNER, &result, &t)?;
            }

            Ok(result)
        });
        methods.add_method("offset", |_, this, offset: isize| {
            let ffiref = unsafe { this.offset(offset)? };
            Ok(ffiref)
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let ffiref = FfiRef::luaref(lua, this)?;
            Ok(ffiref)
        });
    }
}
