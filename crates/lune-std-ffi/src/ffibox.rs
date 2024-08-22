#![allow(clippy::cargo_common_metadata)]

// It is an untyped, sized memory area that Lua can manage.
// This area is safe within Lua. Operations have their boundaries checked.
// It is basically intended to implement passing a pointed space to the outside.
// It also helps you handle data that Lua cannot handle.
// Depending on the type, operations such as sum, mul, and mod may be implemented.
// There is no need to enclose all data in a box;
// rather, it creates more heap space, so it should be used appropriately
// where necessary.

use super::association::set_association;
use super::ffiref::FfiRef;
use core::ffi::c_void;
use mlua::prelude::*;
use std::boxed::Box;

const BOX_REF_INNER: &str = "__box_ref";

pub struct FfiBox(Box<[u8]>);

impl FfiBox {
    pub fn new(size: usize) -> Self {
        Self(vec![0u8; size].into_boxed_slice())
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    // pub fn copy(&self, target: &mut FfiBox) {}

    pub fn get_ptr(&self) -> *mut c_void {
        self.0.as_ptr() as *mut c_void
    }

    // bad naming. i have no idea what should i use
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiBox>()?;

        let luaref = lua.create_userdata(FfiRef::new(target.get_ptr()))?;

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
        methods.add_method_mut("zero", |_, this, ()| {
            this.zero();
            Ok(())
        });
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let luaref = FfiBox::luaref(lua, this)?;
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
