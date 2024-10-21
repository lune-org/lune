#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod data;
mod ffi;

use crate::{
    c::export as c_export,
    data::{create_nullptr, BoxData, LibData, RefData},
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua)?
        .with_function("nullRef", |lua, ()| create_nullptr(lua))?
        .with_function("box", |_lua, size: usize| Ok(BoxData::new(size)))?
        .with_function("open", |_lua, name: String| LibData::new(name))?
        .with_function("uninitRef", |_lua, ()| Ok(RefData::new_uninit()))?
        .with_function("isInteger", |_lua, num: LuaValue| Ok(num.is_integer()))?
        .with_value("c", c_export(lua)?)?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debugAssociation", |lua, str: String| {
        println!("WARNING: ffi.debug_associate is GC debug function, which only works for debug build. Do not use this function in production level codes.");
        ffi::association::get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
