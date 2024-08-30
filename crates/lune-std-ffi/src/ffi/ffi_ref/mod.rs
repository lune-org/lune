use std::ptr;

use mlua::prelude::*;

use super::{
    association_names::REF_INNER,
    ffi_association::{get_association, set_association},
    NativeDataHandle,
};

mod bounds;
mod flags;

pub use self::{
    bounds::{FfiRefBounds, UNSIZED_BOUNDS},
    flags::{FfiRefFlag, FfiRefFlagList},
};

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRef {
    ptr: *mut (),
    pub flags: FfiRefFlagList,
    pub boundary: FfiRefBounds,
}

impl FfiRef {
    pub fn new(ptr: *mut (), flags: FfiRefFlagList, range: FfiRefBounds) -> Self {
        Self {
            ptr,
            flags,
            boundary: range,
        }
    }

    // Make FfiRef from ref
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiRef>()?;
        let mut flags = target.flags.clone();

        // FIXME:
        // We cannot dereference ref which created by lua, in lua
        flags.set_dereferenceable(false);

        let luaref = lua.create_userdata(FfiRef::new(
            ptr::from_ref(&target.ptr) as *mut (),
            flags,
            FfiRefBounds {
                below: 0,
                above: size_of::<usize>(),
            },
        ))?;

        // If the ref holds a box, make sure the new ref also holds the box by holding ref
        set_association(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    pub fn get_ptr(&self) -> *mut () {
        self.ptr
    }

    pub unsafe fn deref(&self) -> LuaResult<Self> {
        self.flags
            .is_dereferenceable()
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
            self.flags.clone(),
            UNSIZED_BOUNDS,
        ))
    }

    pub fn is_nullptr(&self) -> bool {
        self.ptr as usize == 0
    }

    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        self.flags
            .is_offsetable()
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
            self.flags.clone(),
            boundary,
        ))
    }
}

impl NativeDataHandle for FfiRef {
    fn check_boundary(&self, offset: isize, size: usize) -> bool {
        self.boundary.check_sized(offset, size)
    }
    fn checek_writable(&self, userdata: &LuaAnyUserData, offset: isize, size: usize) -> bool {
        self.flags.is_writable()
    }
    // TODO: if ref points box , check box too
    fn check_readable(&self, userdata: &LuaAnyUserData, offset: isize, size: usize) -> bool {
        self.flags.is_readable()
    }
    unsafe fn get_pointer(&self, offset: isize) -> *mut () {
        self.get_ptr().byte_offset(offset)
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
        FfiRefFlagList::zero(),
        // usize::MAX means that nullptr is can be 'any' pointer type
        // We check size of inner data. give ffi.box(1):ref() as argument which typed as i32:ptr() will fail,
        // throw lua error
        UNSIZED_BOUNDS,
    ))
}
