use std::ptr;

use mlua::prelude::*;

use super::{
    association_names::REF_INNER,
    bit_mask::u8_test,
    ffi_association::{get_association, set_association},
    NativeData,
};

mod bounds;
mod flag;

pub use self::{
    bounds::{FfiRefBounds, UNSIZED_BOUNDS},
    flag::FfiRefFlag,
};

// Box:ref():ref() should not be able to modify, Only for external
const BOX_REF_REF_FLAGS: u8 = 0;

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRef {
    ptr: *mut (),
    pub flags: u8,
    pub boundary: FfiRefBounds,
}

impl FfiRef {
    pub fn new(ptr: *mut (), flags: u8, boundary: FfiRefBounds) -> Self {
        Self {
            ptr,
            flags,
            boundary,
        }
    }

    pub fn new_uninit() -> Self {
        Self {
            ptr: ptr::null_mut(),
            flags: FfiRefFlag::Uninit.value(),
            boundary: UNSIZED_BOUNDS,
        }
    }

    // Make FfiRef from ref
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiRef>()?;

        let luaref = lua.create_userdata(FfiRef::new(
            ptr::from_ref(&target.ptr) as *mut (),
            BOX_REF_REF_FLAGS,
            FfiRefBounds {
                below: 0,
                above: size_of::<usize>(),
            },
        ))?;

        // If the ref holds a box, make sure the new ref also holds the box by holding ref
        set_association(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    pub unsafe fn deref(&self) -> LuaResult<Self> {
        u8_test(self.flags, FfiRefFlag::Dereferenceable.value())
            .then_some(())
            .ok_or(LuaError::external("This pointer is not dereferenceable."))?;

        self.boundary
            .check_sized(0, size_of::<usize>())
            .then_some(())
            .ok_or(LuaError::external(
                "Offset is out of bounds. Dereferencing pointer requires size of usize",
            ))?;

        // FIXME flags
        Ok(Self::new(
            *self.ptr.cast::<*mut ()>(),
            self.flags,
            UNSIZED_BOUNDS,
        ))
    }

    pub fn is_nullptr(&self) -> bool {
        self.ptr as usize == 0
    }

    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        u8_test(self.flags, FfiRefFlag::Offsetable.value())
            .then_some(())
            .ok_or(LuaError::external("This pointer is not offsetable."))?;

        // Check boundary, if exceed, return error
        self.boundary
            .check_boundary(offset)
            .then_some(())
            .ok_or_else(|| {
                LuaError::external(format!(
                    "Offset is out of bounds. high: {}, low: {}. offset got {}",
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

impl NativeData for FfiRef {
    fn check_boundary(&self, offset: isize, size: usize) -> bool {
        self.boundary.check_sized(offset, size)
    }
    unsafe fn get_pointer(&self, offset: isize) -> *mut () {
        self.ptr.byte_offset(offset)
    }
    fn is_readable(&self) -> bool {
        u8_test(self.flags, FfiRefFlag::Readable.value())
    }
    fn is_writable(&self) -> bool {
        u8_test(self.flags, FfiRefFlag::Writable.value())
    }
}

impl LuaUserData for FfiRef {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("deref", |lua, this: LuaAnyUserData| {
            let inner = get_association(lua, REF_INNER, &this)?;
            let ffiref = this.borrow::<FfiRef>()?;
            let result = lua.create_userdata(unsafe { ffiref.deref()? })?;

            if let Some(t) = inner {
                // if let Some(u) = get_association(lua, regname, value) {}
                set_association(lua, REF_INNER, &result, &t)?;
            }

            Ok(result)
        });
        methods.add_function("offset", |lua, (this, offset): (LuaAnyUserData, isize)| {
            let ffiref = unsafe { this.borrow::<FfiRef>()?.offset(offset)? };
            let userdata = lua.create_userdata(ffiref)?;

            // If the ref holds a box, make sure the new ref also holds the box
            if let Some(t) = get_association(lua, REF_INNER, &this)? {
                set_association(lua, REF_INNER, &userdata, t)?;
            }

            Ok(userdata)
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let ffiref = FfiRef::luaref(lua, this)?;
            Ok(ffiref)
        });
        methods.add_method("isNullptr", |_, this, ()| Ok(this.is_nullptr()));
    }
}

pub fn create_nullptr(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    // https://en.cppreference.com/w/cpp/types/nullptr_t
    lua.create_userdata(FfiRef::new(
        ptr::null_mut::<()>().cast(),
        0,
        // usize::MAX means that nullptr is can be 'any' pointer type
        // We check size of inner data. give ffi.box(1):ref() as argument which typed as i32:ptr() will fail,
        // throw lua error
        UNSIZED_BOUNDS,
    ))
}
