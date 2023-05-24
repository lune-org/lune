use mlua::prelude::*;

mod require;
mod require_waker;

use crate::builtins::{self, top_level};

const BUILTINS_AS_GLOBALS: &[&str] = &["fs", "net", "process", "stdio", "task"];

pub fn create(lua: &'static Lua, args: Vec<String>) -> LuaResult<()> {
    // Create all builtins
    let builtins = vec![
        ("fs", builtins::fs::create(lua)?),
        ("net", builtins::net::create(lua)?),
        ("process", builtins::process::create(lua, args)?),
        ("serde", builtins::serde::create(lua)?),
        ("stdio", builtins::stdio::create(lua)?),
        ("task", builtins::task::create(lua)?),
        #[cfg(feature = "roblox")]
        ("roblox", builtins::roblox::create(lua)?),
    ];

    // TODO: Remove this when we have proper LSP support for custom
    // require types and no longer need to have builtins as globals
    let lua_globals = lua.globals();
    for name in BUILTINS_AS_GLOBALS {
        let builtin = builtins.iter().find(|(gname, _)| gname == name).unwrap();
        lua_globals.set(*name, builtin.1.clone())?;
    }

    // Create our importer (require) with builtins
    let require_fn = require::create(lua, builtins)?;

    // Create all top-level globals
    let globals = vec![
        ("require", require_fn),
        ("print", lua.create_function(top_level::print)?),
        ("warn", lua.create_function(top_level::warn)?),
        ("error", lua.create_function(top_level::error)?),
    ];

    // Set top-level globals
    for (name, global) in globals {
        lua_globals.set(name, global)?;
    }

    Ok(())
}
