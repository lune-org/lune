use mlua::prelude::*;

mod fs;
mod net;
mod process;
mod require;
mod stdio;
mod task;
mod top_level;

pub fn create(lua: &'static Lua, args: Vec<String>) -> LuaResult<()> {
    // Create all builtins
    let builtins = vec![
        ("fs", fs::create(lua)?),
        ("net", net::create(lua)?),
        ("process", process::create(lua, args)?),
        ("stdio", stdio::create(lua)?),
        ("task", task::create(lua)?),
    ];

    // TODO: Remove this when we have proper LSP support for custom require types
    let lua_globals = lua.globals();
    for (name, builtin) in &builtins {
        lua_globals.set(*name, builtin.clone())?;
    }

    // Create our importer (require) with builtins
    let require_fn = require::create(lua, builtins)?;

    // Create all top-level globals
    let globals = vec![
        ("require", require_fn),
        ("print", lua.create_function(top_level::top_level_print)?),
        ("warn", lua.create_function(top_level::top_level_warn)?),
        ("error", lua.create_function(top_level::top_level_error)?),
        (
            "printinfo",
            lua.create_function(top_level::top_level_printinfo)?,
        ),
    ];

    // Set top-level globals and seal them
    for (name, global) in globals {
        lua_globals.set(name, global)?;
    }
    lua_globals.set_readonly(true);

    Ok(())
}
