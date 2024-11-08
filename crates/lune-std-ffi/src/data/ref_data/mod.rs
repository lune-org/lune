use std::{mem::ManuallyDrop, ptr};

use mlua::prelude::*;

use super::helper::method_provider;
use crate::{
    data::association_names::REF_INNER,
    ffi::{association, bit_mask::*, FfiData},
};

mod bounds;
mod flag;

pub use self::{
    bounds::{RefBounds, UNSIZED_BOUNDS},
    flag::RefFlag,
};

// Box:ref():ref() should not be able to modify, Only for external
const BOX_REF_REF_FLAGS: u8 = 0;
// | FfiRefFlag::Writable.value()
// | FfiRefFlag::Readable.value()
// | FfiRefFlag::Dereferenceable.value()
// | FfiRefFlag::Offsetable.value()
// | FfiRefFlag::Function.value();

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct RefData {
    ptr: ManuallyDrop<Box<*mut ()>>,
    pub flags: u8,
    pub boundary: RefBounds,
}

impl RefData {
    pub fn new(ptr: *mut (), flags: u8, boundary: RefBounds) -> Self {
        Self {
            ptr: ManuallyDrop::new(Box::new(ptr)),
            flags,
            boundary,
        }
    }

    // Make FfiRef from ref
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<RefData>()?;

        let luaref = lua.create_userdata(RefData::new(
            ptr::from_ref(&**target.ptr) as *mut (),
            BOX_REF_REF_FLAGS,
            RefBounds {
                below: 0,
                above: size_of::<usize>(),
            },
        ))?;

        // If the ref holds a box, make sure the new ref also holds the box by holding ref
        association::set(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    pub unsafe fn deref(&self) -> LuaResult<Self> {
        if !u8_test(self.flags, RefFlag::Dereferenceable.value()) {
            return Err(LuaError::external("Reference is not dereferenceable"));
        }

        if !self.boundary.check_sized(0, size_of::<usize>()) {
            return Err(LuaError::external(
                "Offset out of bounds",
            ));
        }

        // FIXME flags
        Ok(Self::new(
            *self.ptr.cast::<*mut ()>(),
            self.flags,
            UNSIZED_BOUNDS,
        ))
    }

    pub fn is_nullptr(&self) -> bool {
        // * ManuallyDrop wrapper
        // * Box wrapper
        (**self.ptr) as usize == 0
    }

    pub fn leak(&mut self) {
        self.flags = u8_set(self.flags, RefFlag::Leaked.value(), true);
    }

    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        u8_test(self.flags, RefFlag::Offsetable.value())
            .then_some(())
            .ok_or_else(|| LuaError::external("Reference is not offsetable"))?;

        // Check boundary, if exceed, return error
        self.boundary
            .check_boundary(offset)
            .then_some(())
            .ok_or_else(|| {
                LuaError::external(format!(
                    "Offset out of bounds (high: {}, low: {}, got {})",
                    self.boundary.above, self.boundary.below, offset
                ))
            })?;

        let boundary = self.boundary.offset(offset);

        // TODO
        Ok(Self::new(
            self.ptr.byte_offset(offset),
            self.flags,
            boundary,
        ))
    }
}

impl Drop for RefData {
    fn drop(&mut self) {
        if u8_test_not(self.flags, RefFlag::Leaked.value()) {
            unsafe { ManuallyDrop::drop(&mut self.ptr) };
        }
    }
}

impl FfiData for RefData {
    #[inline]
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        self.boundary.check_sized(offset, size)
    }
    #[inline]
    unsafe fn get_inner_pointer(&self) -> *mut () {
        **self.ptr
    }
    #[inline]
    fn is_readable(&self) -> bool {
        u8_test(self.flags, RefFlag::Readable.value())
    }
    #[inline]
    fn is_writable(&self) -> bool {
        u8_test(self.flags, RefFlag::Writable.value())
    }
}

impl LuaUserData for RefData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        method_provider::provide_copy_from(methods);

        // FIXME:
        methods.add_function("deref", |lua, this: LuaAnyUserData| {
            let inner = association::get(lua, REF_INNER, &this)?;
            let ffiref = this.borrow::<RefData>()?;
            let result = lua.create_userdata(unsafe { ffiref.deref()? })?;

            if let Some(t) = inner {
                // if let Some(u) = association::get(lua, regname, value) {}
                association::set(lua, REF_INNER, &result, &t)?;
            }

            Ok(result)
        });
        methods.add_function("offset", |lua, (this, offset): (LuaAnyUserData, isize)| {
            let ffiref = unsafe { this.borrow::<RefData>()?.offset(offset)? };
            let userdata = lua.create_userdata(ffiref)?;

            // If the ref holds a box, make sure the new ref also holds the box
            if let Some(t) = association::get(lua, REF_INNER, &this)? {
                association::set(lua, REF_INNER, &userdata, t)?;
            }

            Ok(userdata)
        });
        methods.add_function_mut("leak", |lua, this: LuaAnyUserData| {
            this.borrow_mut::<RefData>()?.leak();
            RefData::luaref(lua, this)
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            RefData::luaref(lua, this)
        });
        methods.add_method("isNull", |_, this, ()| Ok(this.is_nullptr()));
    }
}

pub fn create_nullref(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_userdata(RefData::new(
        ptr::null_mut::<()>().cast(),
        0,
        // usize::MAX means that nullptr is can be 'any' pointer type
        UNSIZED_BOUNDS,
    ))
}
