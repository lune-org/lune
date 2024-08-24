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

use core::ffi::c_void;
use mlua::prelude::*;

use super::association_names::BOX_REF_INNER;
use super::ffi_association::set_association;
use super::ffi_ref::FfiRange;
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

    pub fn size(&self) -> usize {
        self.0.len()
    }

    // pub fn copy(&self, target: &mut FfiBox) {}

    pub fn get_ptr(&self) -> *mut c_void {
        self.0.as_ptr() as *mut c_void
    }

    pub fn stringify(&self) -> String {
        let mut buff = String::from(" ");
        for i in &self.0 {
            buff.push_str(i.to_string().as_str());
            buff.push_str(", ");
        }
        buff.pop();
        buff.pop();
        buff.push(' ');
        buff
    }

    pub fn binary_print(&self) -> String {
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

    // bad naming. i have no idea what should i use
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiBox>()?;
        let ptr = if let Some(t) = offset {
            if t < 0 || t >= (target.size() as isize) {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. box.size: {}. offset got {}",
                    target.size(),
                    t
                )));
            }
            unsafe { target.get_ptr().offset(t) }
        } else {
            target.get_ptr()
        };

        let luaref = lua.create_userdata(FfiRef::new(
            ptr,
            Some(FfiRange {
                low: 0,
                high: target.size() as isize,
            }),
        ))?;

        set_association(lua, BOX_REF_INNER, luaref.clone(), this.clone())?;

        Ok(luaref)
    }

    pub fn zero(&mut self) {
        self.0.fill(0u8);
    }
}

impl LuaUserData for FfiBox {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
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
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(this.binary_print())
        });
    }
}
