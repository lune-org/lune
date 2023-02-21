use mlua::prelude::*;

use crate::{
    lua::stdio::formatting::{format_label, pretty_format_multi_value},
    lua::table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    // HACK: We need to preserve the default behavior of the
    // print and error functions, for pcall and such, which
    // is really tricky to do from scratch so we will just
    // proxy the default print and error functions here
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
            let trace: LuaFunction = lua.named_registry_value("dbg.trace")?;
            error.call((
                LuaError::CallbackError {
                    traceback: format!("override traceback:{}", trace.call::<_, String>(())?),
                    cause: LuaError::external(format!(
                        "{}\n{}",
                        format_label("error"),
                        pretty_format_multi_value(&arg.to_lua_multi(lua)?)?
                    ))
                    .into(),
                },
                level,
            ))?;
            Ok(())
        })?
        // TODO: Add an override for tostring that formats errors in a nicer way
        .build_readonly()
}
