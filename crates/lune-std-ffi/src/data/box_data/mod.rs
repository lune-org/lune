use std::{alloc, alloc::Layout, boxed::Box, mem::ManuallyDrop, ptr};

use mlua::prelude::*;

use crate::{
    data::{association_names::REF_INNER, RefData, RefDataBounds, RefDataFlag},
    ffi::{association, bit_mask::*, FfiData},
};

mod flag;

pub use self::flag::BoxDataFlag;

// Ref which created by lua should not be dereferenceable,
const BOX_REF_FLAGS: u8 =
    RefDataFlag::Readable.value() | RefDataFlag::Writable.value() | RefDataFlag::Offsetable.value();

// It is an untyped, sized memory area that Lua can manage.
// This area is safe within Lua. Operations have their boundaries checked.
// It is basically intended to implement passing a pointed space to the outside.
// It also helps you handle data that Lua cannot handle.
// Depending on the type, operations such as sum, mul, and mod may be implemented.
// There is no need to enclose all data in a box;
// rather, it creates more heap space, so it should be used appropriately
// where necessary.

pub struct BoxData {
    flags: u8,
    data: ManuallyDrop<Box<[u8]>>,
}

const FFI_BOX_PRINT_MAX_LENGTH: usize = 1024;

impl BoxData {
    // For efficiency, it is initialized non-zeroed.
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

    // pub fn copy(&self, target: &mut FfiBox) {}

    pub fn stringify(&self) -> String {
        if self.size() > FFI_BOX_PRINT_MAX_LENGTH * 2 {
            // FIXME
            // Todo: if too big, print as another format
            return String::from("exceed");
        }
        let mut buff: String = String::with_capacity(self.size() * 2);
        for value in self.data.iter() {
            buff.push_str(format!("{:x}", value.to_be()).as_str());
        }
        buff
    }

    pub fn leak(&mut self) {
        self.flags = u8_set(self.flags, BoxDataFlag::Leaked.value(), true);
    }

    // Make FfiRef from box, with boundary checking
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<BoxData>()?;
        let mut bounds = RefDataBounds::new(0, target.size());
        let mut ptr = unsafe { target.get_pointer() };

        // Calculate offset
        if let Some(t) = offset {
            if !bounds.check_boundary(t) {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. box.size: {}. offset got {}",
                    target.size(),
                    t
                )));
            }
            ptr = unsafe { ptr.byte_offset(t) };
            bounds = bounds.offset(t);
        }

        let luaref = lua.create_userdata(RefData::new(ptr.cast(), BOX_REF_FLAGS, bounds))?;

        // Makes box alive longer then ref
        association::set(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    pub unsafe fn drop(&mut self) {
        ManuallyDrop::drop(&mut self.data);
    }

    // Fill every field with 0
    pub fn zero(&mut self) {
        self.data.fill(0);
    }

    // Get size of box
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Drop for BoxData {
    fn drop(&mut self) {
        if u8_test_not(self.flags, BoxDataFlag::Leaked.value()) {
            unsafe { self.drop() };
        }
    }
}

impl FfiData for BoxData {
    fn check_boundary(&self, offset: isize, size: usize) -> bool {
        if offset < 0 {
            return false;
        }
        self.size() - (offset as usize) >= size
    }
    unsafe fn get_pointer(&self) -> *mut () {
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
        fields.add_field_method_get("size", |_, this| Ok(this.size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // For convenience, :zero returns self.
        methods.add_function_mut("zero", |_, this: LuaAnyUserData| {
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
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.stringify()));
    }
}
