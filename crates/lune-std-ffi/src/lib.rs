#![allow(clippy::cargo_common_metadata)]

use std::ffi::c_void;

use libc::free;
use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod data;
mod ffi;

use crate::{
    c::{export_c, export_fixed_types},
    data::{create_nullref, BoxData, GetFfiData, LibData},
};

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `ffi` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua.clone())?
        .with_function("nullRef", |lua, ()| create_nullref(lua))?
        .with_function("box", |_lua, size: usize| Ok(BoxData::new(size)))?
        .with_function("open", |_lua, name: String| LibData::new(name))?
        .with_function("isInteger", |_lua, num: LuaValue| Ok(num.is_integer()))?
        .with_function("free", |_lua, data: LuaAnyUserData| {
            unsafe { free(data.get_ffi_data()?.get_inner_pointer().cast::<c_void>()) };
            Ok(())
        })?
        .with_values(export_fixed_types(&lua)?)?
        .with_value("c", export_c(&lua)?)?;

    result.build_readonly()
}
