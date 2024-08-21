#![allow(clippy::cargo_common_metadata)]

use super::associate::set_associate;
use super::luaref::LuaRef;
use core::ffi::c_void;
use mlua::prelude::*;
use std::boxed::Box;

// use std::borrow::{Borrow, BorrowMut};
// use std::ops::Bound;
// use std::{mem, ptr, slice};

// use core::ffi::c_void;
// use libffi::middle::{Cif, Type};
// use libffi::raw::{ffi_cif, ffi_ptrarray_to_raw};

const BOX_REF_INNER: &str = "__box_ref";

pub struct LuaBox(Box<[u8]>);

impl LuaBox {
    pub fn new(size: usize) -> Self {
        Self(vec![0u8; size].into_boxed_slice())
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn copy(&self, target: &mut LuaBox) {}

    pub fn get_ptr(&self) -> *mut c_void {
        self.0.as_ptr() as *mut c_void
    }

    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<LuaBox>()?;

        let luaref = lua.create_userdata(LuaRef::new(target.get_ptr()))?;

        set_associate(lua, BOX_REF_INNER, luaref.clone(), this.clone())?;

        Ok(luaref)
    }

    pub fn zero(&mut self) {
        self.0.fill(0u8);
    }
}

impl LuaUserData for LuaBox {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("zero", |_, this, ()| {
            this.zero();
            Ok(())
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let luaref = LuaBox::luaref(lua, this)?;
            Ok(luaref)
        });
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.size()));
        methods.add_meta_method(LuaMetaMethod::ToString, |lua, this, ()| {
            dbg!(&this.0.len());
            let mut buff = String::from("[ ");
            for i in &this.0 {
                buff.push_str(i.to_owned().to_string().as_str());
                buff.push_str(", ");
            }
            buff.pop();
            buff.pop();
            buff.push_str(" ]");
            let luastr = lua.create_string(buff.as_bytes())?;
            Ok(luastr)
        });
    }
}
