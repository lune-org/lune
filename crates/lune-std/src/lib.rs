#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

mod global;
mod globals;
mod library;
mod require;

pub use self::global::LuneStandardGlobal;
pub use self::globals::version::set_global_version;
pub use self::library::LuneStandardLibrary;

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
        let module = library.module(lua.clone())?;
        lua.register_module(&alias, module)?;
    }
    Ok(())
}
