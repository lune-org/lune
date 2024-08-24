use core::ffi::c_void;
use std::ptr;

use mlua::prelude::*;

use super::association_names::REF_INNER;
use super::ffi_association::set_association;

// A referenced space. It is possible to read and write through types.
// This operation is not safe. This may cause a memory error in Lua
// if use it incorrectly.
// If it references an area managed by Lua,
// the box will remain as long as this reference is alive.

pub struct FfiRange {
    pub(crate) high: isize,
    pub(crate) low: isize,
}

pub struct FfiRef {
    ptr: *mut c_void,
    range: Option<FfiRange>,
}

impl FfiRef {
    pub fn new(ptr: *mut c_void, range: Option<FfiRange>) -> Self {
        Self { ptr, range }
    }

    // bad naming. i have no idea what should i use
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiRef>()?;

        let luaref = lua.create_userdata(FfiRef::new(
            ptr::from_ref(&target.ptr) as *mut c_void,
            Some(FfiRange {
                low: 0,
                high: size_of::<usize>() as isize,
            }),
        ))?;

        set_association(lua, REF_INNER, luaref.clone(), this.clone())?;

        Ok(luaref)
    }

    pub fn get_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub unsafe fn deref(&self) -> Self {
        Self::new(*self.ptr.cast::<*mut c_void>(), None)
    }

    pub unsafe fn offset(&self, offset: isize) -> LuaResult<Self> {
        let range = if let Some(ref t) = self.range {
            let high = t.high - offset;
            let low = t.low - offset;

            if low > 0 || high < 0 {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. low: {}, high: {}. offset got {}",
                    t.low, t.high, offset
                )));
            }

            Some(FfiRange { high, low })
        } else {
            None
        };

        Ok(Self::new(self.ptr.offset(offset), range))
    }
}

impl LuaUserData for FfiRef {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("deref", |_, this, ()| {
            let ffiref = unsafe { this.deref() };
            Ok(ffiref)
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
