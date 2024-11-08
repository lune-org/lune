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
const REF_REF_FLAGS: u8 = 0;

const DEREF_REF_FLAG: u8 = RefFlag::Dereferenceable.value()
    | RefFlag::Function.value()
    | RefFlag::Offsetable.value()
    | RefFlag::Readable.value()
    | RefFlag::Writable.value();

// A referenced memory address box. Possible to read and write through types.
pub struct RefData {
    ptr: ManuallyDrop<Box<*mut ()>>,
    pub(crate) flags: u8,
    boundary: RefBounds,
}

impl RefData {
    pub fn new(ptr: *mut (), flags: u8, boundary: RefBounds) -> Self {
        Self {
            ptr: ManuallyDrop::new(Box::new(ptr)),
            flags,
            boundary,
        }
    }

    // Create reference of this reference box
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<RefData>()?;

        let luaref = lua.create_userdata(RefData::new(
            ptr::from_ref(&**target.ptr) as *mut (),
            REF_REF_FLAGS,
            RefBounds {
                below: 0,
                above: size_of::<usize>(),
            },
        ))?;

        // Make new reference live longer then this reference
        association::set(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    // Dereference this reference
    pub unsafe fn dereference(&self) -> LuaResult<Self> {
        // Check dereferenceable
        if !u8_test(self.flags, RefFlag::Dereferenceable.value()) {
            return Err(LuaError::external("Reference is not dereferenceable"));
        }

        // Check boundary
        if !self.boundary.check_sized(0, size_of::<usize>()) {
            return Err(LuaError::external("Out of bounds"));
        }

        Ok(Self::new(
            *self.ptr.cast::<*mut ()>(),
            DEREF_REF_FLAG,
            UNSIZED_BOUNDS,
        ))
    }

    pub fn is_null(&self) -> bool {
        // * ManuallyDrop wrapper
        // * Box wrapper
        (**self.ptr) as usize == 0
    }

    pub fn leak(&mut self) {
        self.flags = u8_set(self.flags, RefFlag::Leaked.value(), true);
    }

    // Create new reference with specific offset from this reference
    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        // Check offsetable
        if u8_test_not(self.flags, RefFlag::Offsetable.value()) {
            return Err(LuaError::external("Reference is not offsetable"));
        }

        // Check boundary
        if !self.boundary.check_boundary(offset) {
            return Err(LuaError::external(format!(
                "Offset out of bounds (high: {}, low: {}, got {})",
                self.boundary.above, self.boundary.below, offset
            )));
        }

        let boundary = self.boundary.offset(offset);
        Ok(Self::new(
            self.ptr.byte_offset(offset),
            u8_set(self.flags, RefFlag::Leaked.value(), false),
            boundary,
        ))
    }

    // Stringify for pretty-print, with hex format address
    pub fn stringify(&self) -> String {
        format!("{:x}", **self.ptr as usize)
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

        methods.add_method("deref", |_lua, this, ()| unsafe { this.dereference() });
        methods.add_function("offset", |lua, (this, offset): (LuaAnyUserData, isize)| {
            let ffiref = unsafe { this.borrow::<RefData>()?.offset(offset)? };
            let userdata = lua.create_userdata(ffiref)?;

            // If the ref holds a box or reference, make sure the new ref also holds it
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
        methods.add_method("isNull", |_lua, this, ()| Ok(this.is_null()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_lua, this, ()| {
            Ok(this.stringify())
        });
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
