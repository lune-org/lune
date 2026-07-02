#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

mod global;
mod globals;
mod library;
mod require;
mod unsafe_library;

pub use self::global::LuneStandardGlobal;
pub use self::globals::version::set_global_version;
pub use self::library::LuneStandardLibrary;
pub use self::unsafe_library::{get_unsafe_library_enabled, set_unsafe_library_enabled};

/**
    Injects all standard globals into the given Lua state / VM.

    This **does not** include standard libraries - see `inject_std`.

    # Errors

    Errors when out of memory, or if *default* Lua globals are missing.
*/
pub fn inject_globals(lua: Lua) -> LuaResult<()> {
    for global in LuneStandardGlobal::ALL {
        lua.globals()
            .set(global.name(), global.create(lua.clone())?)?;
    }
    Ok(())
}

/**
    Injects all standard libraries into the given Lua state / VM.

    # Errors

    Errors when out of memory, or if *default* Lua globals are missing.
*/
pub fn inject_std(lua: Lua) -> LuaResult<()> {
    for library in LuneStandardLibrary::ALL {
        let alias = format!("@lune/{}", library.name());
        let module = if library.is_unsafe() && !get_unsafe_library_enabled(&lua) {
            create_unsafe_stub_module(&lua, library.name())?
        } else {
            library.module(lua.clone())?
        };
        lua.register_module(&alias, module)?;
    }
    Ok(())
}

/**
    Creates a stub module for an unsafe library that has not been enabled.

    Requiring the library will still work, but any usage of it will error
    with a message explaining how to enable the unsafe library instead.
*/
fn create_unsafe_stub_module(lua: &Lua, name: &'static str) -> LuaResult<LuaTable> {
    let make_error = lua.create_function(move |_, ()| -> LuaResult<()> {
        Err(LuaError::external(format!(
            "The `{name}` standard library is unsafe and requires \
            the unsafe library flag (--unsafe) to be enabled"
        )))
    })?;

    let meta = lua.create_table()?;
    meta.set("__index", make_error.clone())?;
    meta.set("__newindex", make_error.clone())?;
    meta.set("__call", make_error)?;
    meta.set("__metatable", false)?;

    let stub = lua.create_table()?;
    stub.set_metatable(Some(meta))?;
    Ok(stub)
}
