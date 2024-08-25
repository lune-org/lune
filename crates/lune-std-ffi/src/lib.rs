#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod ffi_association;
mod c_arr;
mod c_fn;
mod c_helper;
mod c_ptr;
mod c_string;
mod c_struct;
mod c_type;
mod ffi_box;
mod ffi_helper;
mod ffi_lib;
mod ffi_platform;
mod ffi_raw;
mod ffi_ref;

use crate::ffi_association::get_table;
use crate::c_fn::CFn;
use crate::c_struct::CStruct;
use crate::c_type::create_all_types;
use crate::ffi_box::FfiBox;
use crate::ffi_lib::FfiLib;
use crate::ffi_platform::get_platform_value;

// Converts ffi status into &str
pub const FFI_STATUS_NAMES: [&str; 4] = [
    "ffi_status_FFI_OK",
    "ffi_status_FFI_BAD_TYPEDEF",
    "ffi_status_FFI_BAD_ABI",
    "ffi_status_FFI_BAD_ARGTYPE",
];

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
    pub const BOX_REF_INNER: &str = "__box_ref";
    pub const REF_INNER: &str = "__ref_inner";
}

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
        .with_function("fn", |_, (args, ret): (LuaTable, LuaAnyUserData)| {
            let cfn = CFn::from_lua_table(args, ret)?;
            Ok(cfn)
        })?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
