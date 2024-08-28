#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod ffi;

use crate::{
    c::{create_all_c_types, create_all_types, CFn, CStruct},
    ffi::{create_nullptr, FfiBox, FfiLib},
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let result = TableBuilder::new(lua)?
        .with_values(create_all_types(lua)?)?
        .with_values(create_all_c_types(lua)?)?
        .with_value("nullptr", create_nullptr(lua)?)?
        .with_function("box", |_, size: usize| Ok(FfiBox::new(size)))?
        // TODO: discuss about function name. matching with io.open is better?
        .with_function("dlopen", |_, name: String| {
            let lib = FfiLib::new(name)?;
            Ok(lib)
        })?
        .with_function("struct", |lua, types: LuaTable| {
            let cstruct = CStruct::new_from_lua_table(lua, types)?;
            Ok(cstruct)
        })?
        .with_function("fn", |lua, (args, ret): (LuaTable, LuaAnyUserData)| {
            let cfn = CFn::new_from_lua_table(lua, args, ret)?;
            Ok(cfn)
        })?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        println!("WARNING: ffi.debug_associate is GC debug function, which only works for debug build. Do not use this function in production level codes.");
        crate::ffi::ffi_association::get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
