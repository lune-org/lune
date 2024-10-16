#![allow(clippy::cargo_common_metadata)]

use ffi::FfiRef;
use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod ffi;
mod libffi_helper;

use crate::{
    c::{export_ctypes, CFunc, CStruct},
    ffi::{create_nullptr, is_integer, FfiBox, FfiLib},
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua)?
        .with_values(export_ctypes(lua)?)?
        .with_value("nullRef", create_nullptr(lua)?)?
        .with_function("box", |_lua, size: usize| Ok(FfiBox::new(size)))?
        .with_function("open", |_lua, name: String| FfiLib::new(name))?
        .with_function("structInfo", |lua, types: LuaTable| {
            CStruct::from_table(lua, types)
        })?
        .with_function("uninitRef", |_lua, ()| Ok(FfiRef::new_uninit()))?
        .with_function("isInteger", |_lua, num: LuaValue| Ok(is_integer(num)))?
        .with_function(
            "funcInfo",
            |lua, (args, ret): (LuaTable, LuaAnyUserData)| CFunc::new_from_table(lua, args, ret),
        )?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        println!("WARNING: ffi.debug_associate is GC debug function, which only works for debug build. Do not use this function in production level codes.");
        crate::ffi::ffi_association::get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
