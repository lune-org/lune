#![allow(clippy::cargo_common_metadata)]

// It is an untyped, sized memory area that Lua can manage.
// This area is safe within Lua. Operations have their boundaries checked.
// It is basically intended to implement passing a pointed space to the outside.
// It also helps you handle data that Lua cannot handle.
// Depending on the type, operations such as sum, mul, and mod may be implemented.
// There is no need to enclose all data in a box;
// rather, it creates more heap space, so it should be used appropriately
// where necessary.

use std::boxed::Box;

use mlua::prelude::*;

use super::association_names::REF_INNER;
use super::ffi_association::set_association;
use super::ffi_bounds::FfiRefBounds;
use super::ffi_ref::FfiRef;

pub struct FfiBox(Box<[u8]>);

impl FfiBox {
    // For efficiency, it is initialized non-zeroed.
    pub fn new(size: usize) -> Self {
        // Create new vector to allocate heap memory. sized with 'size'
        let mut vec_heap = Vec::<u8>::with_capacity(size);

        // It is safe to have a length equal to the capacity
        #[allow(clippy::uninit_vec)]
        unsafe {
            vec_heap.set_len(size);
        }

        Self(vec_heap.into_boxed_slice())
    }

    // pub fn copy(&self, target: &mut FfiBox) {}

    // Todo: if too big, print as another format
    pub fn stringify(&self) -> String {
        let mut buff: String = String::with_capacity(self.size() * 10 - 2);
        for (pos, value) in self.0.iter().enumerate() {
            for i in 0..8 {
                if (value & (1 << i)) == 0 {
                    buff.push('0');
                } else {
                    buff.push('1');
                }
            }
            if pos < self.size() - 1 {
                buff.push_str(", ");
            }
        }
        buff
    }

    // Make FfiRef from box, with boundary checking
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let mut target = this.borrow_mut::<FfiBox>()?;
        let mut bounds = FfiRefBounds::new(0, target.size());
        let mut ptr = target.get_ptr();

        // Calculate offset
        if let Some(t) = offset {
            if !bounds.check(t) {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. box.size: {}. offset got {}",
                    target.size(),
                    t
                )));
            }
            ptr = unsafe { target.get_ptr().byte_offset(t) };
            bounds = bounds.offset(t);
        }

        // Lua should not be able to deref a box that refers to a box managed by Lua.
        // To deref a box space is to allow lua to read any space,
        // which has security issues and is ultimately dangerous.
        // Therefore, box:ref():deref() is not allowed.
        let luaref = lua.create_userdata(FfiRef::new(ptr.cast(), false, Some(bounds)))?;

        // Makes box alive longer then ref
        set_association(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    // Fill every field with 0
    pub fn zero(&mut self) {
        self.0.fill(0u8);
    }

    // Get size of box
    pub fn size(&self) -> usize {
        self.0.len()
    }

    // Get raw ptr
    pub fn get_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}

impl LuaUserData for FfiBox {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // For convenience, :zero returns self.
        methods.add_function_mut("zero", |_, this: LuaAnyUserData| {
            this.borrow_mut::<FfiBox>()?.zero();
            Ok(this)
        });
        methods.add_function(
            "ref",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                let luaref = FfiBox::luaref(lua, this, offset)?;
                Ok(luaref)
            },
        );
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.stringify()));
    }
}
