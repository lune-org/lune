use mlua::prelude::*;

mod require;
mod require_waker;

use crate::lune::builtins::{self, top_level};

pub fn create(lua: &'static Lua, args: Vec<String>) -> LuaResult<()> {
    // Create all builtins
    let builtins = vec![
        ("fs", builtins::fs::create(lua)?),
        ("net", builtins::net::create(lua)?),
        ("process", builtins::process::create(lua, args)?),
        ("serde", builtins::serde::create(lua)?),
        ("stdio", builtins::stdio::create(lua)?),
        ("task", builtins::task::create(lua)?),
        ("luau", builtins::luau::create(lua)?),
        #[cfg(feature = "roblox")]
        ("roblox", builtins::roblox::create(lua)?),
    ];

    // Create our importer (require) with builtins
    let require_fn = require::create(lua, builtins)?;

    // Create all top-level globals
    let globals = vec![
        ("require", require_fn),
        ("print", lua.create_function(top_level::print)?),
        ("warn", lua.create_function(top_level::warn)?),
        ("error", lua.create_function(top_level::error)?),
        ("type", lua.create_function(top_level::proxy_type)?),
        ("typeof", lua.create_function(top_level::proxy_typeof)?),
    ];

    // Set top-level globals
    let lua_globals = lua.globals();
    for (name, global) in globals {
        lua_globals.set(name, global)?;
    }

    Ok(())
}
