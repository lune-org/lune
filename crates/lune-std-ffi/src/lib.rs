#![allow(clippy::cargo_common_metadata)]

use data::RefData;
use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod data;
mod ffi;

use crate::{
    c::{export_ctypes, CFnInfo, CStructInfo},
    data::{create_nullptr, BoxData, LibData},
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua)?
        .with_values(export_ctypes(lua)?)?
        .with_function("nullRef", |lua, ()| create_nullptr(lua))?
        .with_function("box", |_lua, size: usize| Ok(BoxData::new(size)))?
        .with_function("open", |_lua, name: String| LibData::new(name))?
        .with_function("structInfo", |lua, types: LuaTable| {
            CStructInfo::from_table(lua, types)
        })?
        .with_function("uninitRef", |_lua, ()| Ok(RefData::new_uninit()))?
        .with_function("isInteger", |_lua, num: LuaValue| Ok(num.is_integer()))?
        .with_function("fnInfo", |lua, (args, ret): (LuaTable, LuaAnyUserData)| {
            CFnInfo::new_from_table(lua, args, ret)
        })?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        println!("WARNING: ffi.debug_associate is GC debug function, which only works for debug build. Do not use this function in production level codes.");
        ffi::association::get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
