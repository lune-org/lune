use std::slice;

use mlua::prelude::*;

use super::helper::method_provider;
use crate::{
    data::{association_names::REF_INNER, RefBounds, RefData, RefFlag},
    ffi::{association, bit_field::*, FfiData},
};

mod flag;

pub use self::flag::BoxFlag;

const FFI_BOX_PRINT_MAX_LENGTH: usize = 1024;

// Box refs are dereferenceable so pointers stored in the box can be followed
const BOX_REF_FLAGS: u8 = RefFlag::Readable.value()
    | RefFlag::Writable.value()
    | RefFlag::Offsetable.value()
    | RefFlag::Dereferenceable.value();

// Untyped runtime sized memory for luau.
// This operations are safe, have boundaries check.
//
// Allocated with libc::malloc so ffi.free (libc::free) always matches.
pub struct BoxData {
    flags: u8,
    size: usize,
    data: *mut u8,
}

impl BoxData {
    pub fn new(size: usize) -> LuaResult<Self> {
        if size == 0 {
            return Err(LuaError::external(
                "Cannot create a zero-sized box; the size must be greater than 0",
            ));
        }
        let data = unsafe { libc::malloc(size).cast::<u8>() };
        if data.is_null() {
            return Err(LuaError::external(format!(
                "Failed to allocate {size} bytes of memory for the box",
            )));
        }

        Ok(Self {
            flags: 0,
            size,
            data,
        })
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.size) }
    }

    // Stringify for pretty-print, with hex format content
    pub fn stringify(&self) -> String {
        if u8_test(self.flags, BoxFlag::Freed.value()) {
            return String::from("freed");
        }
        if self.size > FFI_BOX_PRINT_MAX_LENGTH {
            return String::from("content length limit exceeded");
        }
        let mut buff: String = String::with_capacity(self.size * 2 + 2);
        buff.push_str("0x");
        for value in self.as_slice() {
            buff.push_str(format!("{value:02x}").as_str());
        }
        buff
    }

    pub fn leak(&mut self) -> LuaResult<()> {
        self.ensure_not_freed()?;
        self.flags = u8_set(self.flags, BoxFlag::Leaked.value(), true);
        Ok(())
    }

    // Free now instead of at GC, and stop the destructor from touching it again
    pub fn free(&mut self) -> LuaResult<()> {
        self.ensure_not_freed()?;
        unsafe { libc::free(self.data.cast()) };
        self.flags = u8_set(self.flags, BoxFlag::Freed.value(), true);
        Ok(())
    }

    fn ensure_not_freed(&self) -> LuaResult<()> {
        if u8_test(self.flags, BoxFlag::Freed.value()) {
            Err(LuaError::external("Box memory has already been freed"))
        } else {
            Ok(())
        }
    }

    // Make FfiRef from box, with boundary check
    pub fn luaref(
        lua: &Lua,
        this: LuaAnyUserData,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData> {
        let target = this.borrow::<BoxData>()?;
        target.ensure_not_freed()?;
        let mut bounds = RefBounds::new(0, target.size());
        let mut ptr = unsafe { target.get_inner_pointer() };

        // Calculate offset
        if let Some(t) = offset {
            if !bounds.check_offset(t) {
                return Err(LuaError::external(format!(
                    "Offset out of bounds (box.size: {}, got {})",
                    target.size(),
                    t
                )));
            }
            ptr = unsafe { ptr.byte_offset(t) };
            bounds = bounds.offset(t);
        }

        let luaref = lua.create_userdata(RefData::new(ptr.cast(), BOX_REF_FLAGS, bounds))?;

        // Make box live longer then ref
        association::set(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    // Fill with zero
    pub fn zero(&mut self) -> LuaResult<()> {
        self.ensure_not_freed()?;
        unsafe { self.data.write_bytes(0, self.size) };
        Ok(())
    }

    // Get size of box
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for BoxData {
    fn drop(&mut self) {
        if u8_test_not(self.flags, BoxFlag::Leaked.value())
            && u8_test_not(self.flags, BoxFlag::Freed.value())
        {
            unsafe { libc::free(self.data.cast()) };
        }
    }
}

impl FfiData for BoxData {
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        if u8_test(self.flags, BoxFlag::Freed.value()) || offset < 0 {
            return false;
        }
        let offset = offset.unsigned_abs();
        offset <= self.size() && self.size() - offset >= size
    }
    #[inline]
    unsafe fn get_inner_pointer(&self) -> *mut () {
        self.data.cast::<()>()
    }
    fn is_readable(&self) -> bool {
        u8_test_not(self.flags, BoxFlag::Freed.value())
    }
    fn is_writable(&self) -> bool {
        u8_test_not(self.flags, BoxFlag::Freed.value())
    }
}

impl LuaUserData for BoxData {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_lua, this| Ok(this.size()));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        method_provider::provide_copy_from(methods);
        method_provider::provide_read_string(methods);
        method_provider::provide_read_c_string(methods);
        method_provider::provide_write_string(methods);

        // For convenience, :zero returns box itself.
        methods.add_function_mut("zero", |_lua, this: LuaAnyUserData| {
            this.borrow_mut::<BoxData>()?.zero()?;
            Ok(this)
        });
        methods.add_function_mut(
            "leak",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                this.borrow_mut::<BoxData>()?.leak()?;
                BoxData::luaref(lua, this, offset)
            },
        );
        methods.add_function(
            "ref",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                BoxData::luaref(lua, this, offset)
            },
        );
        methods.add_meta_method(LuaMetaMethod::ToString, |_lua, this, ()| {
            Ok(this.stringify())
        });
    }
}
