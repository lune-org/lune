#![allow(clippy::cargo_common_metadata)]

use lune_utils::TableBuilder;
use mlua::prelude::*;

mod associate;
mod cfn;
mod cstruct;
mod ctype;
mod dlopen;
mod luabox;
mod luaraw;
mod luaref;

use self::associate::get_table;
use self::cstruct::CStruct;
use self::ctype::create_all_types;
use self::dlopen::LuaLibrary;
use self::luabox::LuaBox;

/**
    Creates the `ffi` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let ctypes = create_all_types(lua)?;
    let result = TableBuilder::new(lua)?
        .with_values(ctypes)?
        .with_function("box", |_, size: usize| Ok(LuaBox::new(size)))?
        .with_function("dlopen", |_, name: String| {
            let lib = LuaLibrary::new(name)?;
            Ok(lib)
        })?
        .with_function("struct", |lua, types: LuaTable| {
            let cstruct = CStruct::from_lua_table(lua, types)?;
            Ok(cstruct)
        })?;

    #[cfg(debug_assertions)]
    let result = result.with_function("debug_associate", |lua, str: String| {
        get_table(lua, str.as_ref())
    })?;

    result.build_readonly()
}
