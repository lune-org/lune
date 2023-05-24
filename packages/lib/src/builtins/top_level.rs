use mlua::prelude::*;

use crate::lua::stdio::formatting::{format_label, pretty_format_multi_value};

// HACK: We need to preserve the default behavior of the
// print and error functions, for pcall and such, which
// is really tricky to do from scratch so we will just
// proxy the default print and error functions here

pub fn print(lua: &Lua, args: LuaMultiValue) -> LuaResult<()> {
    let formatted = pretty_format_multi_value(&args)?;
    let print: LuaFunction = lua.named_registry_value("print")?;
    print.call(formatted)?;
    Ok(())
}

pub fn warn(lua: &Lua, args: LuaMultiValue) -> LuaResult<()> {
    let print: LuaFunction = lua.named_registry_value("print")?;
    print.call(format!(
        "{}\n{}",
        format_label("warn"),
        pretty_format_multi_value(&args)?
    ))?;
    Ok(())
}

pub fn error(lua: &Lua, (arg, level): (LuaValue, Option<u32>)) -> LuaResult<()> {
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
}

// TODO: Add an override for tostring that formats errors in a nicer way
