#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

mod global;
mod globals;
mod library;
mod luaurc;
mod unsafe_library;

pub use self::global::LuneStandardGlobal;
pub use self::globals::version::set_global_version;
pub use self::library::LuneStandardLibrary;
pub use self::unsafe_library::{get_unsafe_library_enabled, set_unsafe_library_enabled};

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
