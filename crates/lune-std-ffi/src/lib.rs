#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod association;
mod carr;
mod cfn;
mod cstring;
mod cstruct;
mod ctype;
mod ffibox;
mod ffilib;
mod ffiraw;
mod ffiref;

use self::association::get_table;
use self::cfn::CFn;
use self::cstruct::CStruct;
use self::ctype::create_all_types;
use self::ffibox::FfiBox;
use self::ffilib::FfiLib;

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let ctypes = create_all_types(lua)?;
    let result = TableBuilder::new(lua)?
        .with_values(ctypes)?
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
