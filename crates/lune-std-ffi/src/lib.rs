#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod c;
mod ffi;

use crate::c::{c_fn::CFn, c_struct::CStruct, create_all_types};
use crate::ffi::{
    ffi_association::get_table, ffi_box::FfiBox, ffi_lib::FfiLib, ffi_platform::get_platform_value,
};

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let ctypes = create_all_types(lua)?;
    let result = TableBuilder::new(lua)?
        .with_values(ctypes)?
        .with_values(get_platform_value())?
        .with_function("box", |_, size: usize| Ok(FfiBox::new(size)))?
        // TODO: discuss about function name. matching with io.open is better?
        .with_function("dlopen", |_, name: String| {
            let lib = FfiLib::new(name)?;
            Ok(lib)
        })?
        .with_function("struct", |lua, types: LuaTable| {
            let cstruct = CStruct::from_lua_table(lua, types)?;
            Ok(cstruct)
        })?
        .with_function("fn", |lua, (args, ret): (LuaTable, LuaAnyUserData)| {
            let cfn = CFn::from_lua_table(lua, args, ret)?;
            Ok(cfn)
        })?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
