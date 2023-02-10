use mlua::prelude::*;

use crate::utils::{
    formatting::{format_label, pretty_format_multi_value},
    table::TableBuilder,
};

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    let globals = lua.globals();
    // HACK: We need to preserve the default behavior of the
    // print and error functions, for pcall and such, which
    // is really tricky to do from scratch so we will just
    // proxy the default print and error functions here
    let print_fn: LuaFunction = globals.raw_get("print")?;
    let error_fn: LuaFunction = globals.raw_get("error")?;
    lua.set_named_registry_value("print", print_fn)?;
    lua.set_named_registry_value("error", error_fn)?;
    TableBuilder::new(lua)?
        .with_function("print", |lua, args: LuaMultiValue| {
            let formatted = pretty_format_multi_value(&args)?;
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(formatted)?;
            Ok(())
        })?
        .with_function("info", |lua, args: LuaMultiValue| {
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(format!(
                "{}\n{}",
                format_label("info"),
                pretty_format_multi_value(&args)?
            ))?;
            Ok(())
        })?
        .with_function("warn", |lua, args: LuaMultiValue| {
            let print: LuaFunction = lua.named_registry_value("print")?;
            print.call(format!(
                "{}\n{}",
                format_label("warn"),
                pretty_format_multi_value(&args)?
            ))?;
            Ok(())
        })?
        .with_function("error", |lua, (arg, level): (LuaValue, Option<u32>)| {
            let error: LuaFunction = lua.named_registry_value("error")?;
            let multi = arg.to_lua_multi(lua)?;
            error.call((
                format!(
                    "{}\n{}",
                    format_label("error"),
                    pretty_format_multi_value(&multi)?
                ),
                level,
            ))?;
            Ok(())
        })?
        .build_readonly()
}
