use std::{
    alloc::{self, Layout},
    boxed::Box,
    mem::ManuallyDrop,
    ptr,
};

use mlua::prelude::*;

use super::helper::method_provider;
use crate::{
    data::{association_names::REF_INNER, RefBounds, RefData, RefFlag},
    ffi::{association, bit_mask::*, FfiData},
};

mod flag;

pub use self::flag::BoxFlag;

const FFI_BOX_PRINT_MAX_LENGTH: usize = 1024;

// Reference which created by lua should not be dereferenceable
const BOX_REF_FLAGS: u8 =
    RefFlag::Readable.value() | RefFlag::Writable.value() | RefFlag::Offsetable.value();

// Untyped runtime sized memory for luau.
// This operations are safe, have boundaries check.
pub struct BoxData {
    flags: u8,
    data: ManuallyDrop<Box<[u8]>>,
}

impl BoxData {
    pub fn new(size: usize) -> Self {
        let slice = unsafe {
            Box::from_raw(ptr::slice_from_raw_parts_mut(
                alloc::alloc(Layout::array::<u8>(size).unwrap()),
                size,
            ))
        };

        Self {
            flags: 0,
            data: ManuallyDrop::new(slice),
        }
    }

    // Stringify for pretty-print, with hex format content
    pub fn stringify(&self) -> String {
        if self.size() > FFI_BOX_PRINT_MAX_LENGTH * 2 {
            return String::from("length limit exceed");
        }
        let mut buff: String = String::with_capacity(self.size() * 2 + 2);
        buff.push_str("0x");
        for value in self.data.iter() {
            buff.push_str(format!("{:x}", value.to_be()).as_str());
        }
        buff
    }

    pub fn leak(&mut self) {
        self.flags = u8_set(self.flags, BoxFlag::Leaked.value(), true);
    }

    // Make FfiRef from box, with boundary check
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<BoxData>()?;
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
    pub fn zero(&mut self) {
        self.data.fill(0);
    }

    // Get size of box
    #[inline]
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Drop for BoxData {
    fn drop(&mut self) {
        if u8_test_not(self.flags, BoxFlag::Leaked.value()) {
            unsafe { ManuallyDrop::drop(&mut self.data) };
        }
    }
}

impl FfiData for BoxData {
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        if offset < 0 {
            return false;
        }
        self.size() - (offset as usize) >= size
    }
    #[inline]
    unsafe fn get_inner_pointer(&self) -> *mut () {
        self.data.as_ptr().cast_mut().cast::<()>()
    }
    fn is_readable(&self) -> bool {
        true
    }
    fn is_writable(&self) -> bool {
        true
    }
}

impl LuaUserData for BoxData {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_lua, this| Ok(this.size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        method_provider::provide_copy_from(methods);
        // For convenience, :zero returns box itself.
        methods.add_function_mut("zero", |_lua, this: LuaAnyUserData| {
            this.borrow_mut::<BoxData>()?.zero();
            Ok(this)
        });
        methods.add_function_mut(
            "leak",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                this.borrow_mut::<BoxData>()?.leak();
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
