#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

mod global;
mod globals;
mod library;

pub use self::global::LuneStandardGlobal;
pub use self::globals::version::set_global_version;
pub use self::library::LuneStandardLibrary;

/**
    Injects all standard globals into the given Lua state / VM.

    This includes all enabled standard libraries, which can
    be used from Lua with `require("@lune/library-name")`.

    # Errors

    Errors when out of memory, or if *default* Lua globals are missing.
*/
pub fn inject_globals(lua: &Lua) -> LuaResult<()> {
    for global in LuneStandardGlobal::ALL {
        lua.globals().set(global.name(), global.create(lua)?)?;
    }
    Ok(())
}
