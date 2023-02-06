mod console;
mod fs;
mod net;
mod process;
mod require;
mod task;

// Global tables

pub use console::create as create_console;
pub use fs::create as create_fs;
pub use net::create as create_net;
pub use process::create as create_process;
pub use require::create as create_require;
pub use task::create as create_task;

// Individual top-level global values

use mlua::prelude::*;

use crate::utils::formatting::pretty_format_multi_value;

pub fn create_top_level(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    // HACK: We need to preserve the default behavior of the
    // print and error functions, for pcall and such, which
    // is really tricky to do from scratch so we will just
    // proxy the default print and error functions here
    let print_fn: LuaFunction = globals.raw_get("print")?;
    let error_fn: LuaFunction = globals.raw_get("error")?;
    lua.set_named_registry_value("print", print_fn)?;
    lua.set_named_registry_value("error", error_fn)?;
    globals.raw_set(
        "print",
        lua.create_function(|lua, args: LuaMultiValue| {
            let formatted = pretty_format_multi_value(&args)?;
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(formatted)?;
            Ok(())
        })?,
    )?;
    globals.raw_set(
        "info",
        lua.create_function(|lua, args: LuaMultiValue| {
            let formatted = pretty_format_multi_value(&args)?;
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(formatted)?;
            Ok(())
        })?,
    )?;
    globals.raw_set(
        "warn",
        lua.create_function(|lua, args: LuaMultiValue| {
            let formatted = pretty_format_multi_value(&args)?;
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(formatted)?;
            Ok(())
        })?,
    )?;
    globals.raw_set(
        "error",
        lua.create_function(|lua, (arg, level): (LuaValue, Option<u32>)| {
            let multi = arg.to_lua_multi(lua)?;
            let formatted = pretty_format_multi_value(&multi)?;
            let error: LuaFunction = lua.named_registry_value("error")?;
            error.call((formatted, level))?;
            Ok(())
        })?,
    )?;
    Ok(())
}
