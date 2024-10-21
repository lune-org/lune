#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod data;
mod ffi;

use crate::{
    c::{export_c, export_fixed_types},
    data::{create_nullref, BoxData, LibData},
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua)?
        .with_values(export_fixed_types(lua)?)?
        .with_function("nullRef", |lua, ()| create_nullref(lua))?
        .with_function("box", |_lua, size: usize| Ok(BoxData::new(size)))?
        .with_function("open", |_lua, name: String| LibData::new(name))?
        .with_function("isInteger", |_lua, num: LuaValue| Ok(num.is_integer()))?
        .with_value("c", export_c(lua)?)?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debugAssociation", |lua, str: String| {
        println!("WARNING: ffi.debug_associate is GC debug function, which only works for debug build. Do not use this function in production level codes.");
        ffi::association::get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
