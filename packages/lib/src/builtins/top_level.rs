use mlua::prelude::*;
use std::io::{self, Write as _};

#[cfg(feature = "roblox")]
use lune_roblox::datatypes::extension::RobloxUserdataTypenameExt;

use crate::lua::{
    stdio::formatting::{format_label, pretty_format_multi_value},
    task::TaskReference,
};

pub fn print(_: &Lua, args: LuaMultiValue) -> LuaResult<()> {
    let formatted = format!("{}\n", pretty_format_multi_value(&args)?);
    let mut stdout = io::stdout();
    stdout.write_all(formatted.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

pub fn warn(_: &Lua, args: LuaMultiValue) -> LuaResult<()> {
    let formatted = format!(
        "{}\n{}",
        format_label("warn"),
        pretty_format_multi_value(&args)?
    );
    let mut stdout = io::stdout();
    stdout.write_all(formatted.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

// HACK: We need to preserve the default behavior of
// the lua error function, for pcall and such, which
// is really tricky to do from scratch so we will
// just proxy the default function here instead

pub fn error(lua: &Lua, (arg, level): (LuaValue, Option<u32>)) -> LuaResult<()> {
    let error: LuaFunction = lua.named_registry_value("error")?;
    let trace: LuaFunction = lua.named_registry_value("dbg.trace")?;
    error.call((
        LuaError::CallbackError {
            traceback: format!("override traceback:{}", trace.call::<_, String>(())?),
            cause: LuaError::external(format!(
                "{}\n{}",
                format_label("error"),
                pretty_format_multi_value(&arg.into_lua_multi(lua)?)?
            ))
            .into(),
        },
        level,
    ))?;
    Ok(())
}

pub fn proxy_type<'lua>(lua: &'lua Lua, value: LuaValue<'lua>) -> LuaResult<LuaString<'lua>> {
    if let LuaValue::UserData(u) = &value {
        if u.is::<TaskReference>() {
            return lua.create_string("thread");
        }
    }
    lua.named_registry_value::<LuaFunction>("type")?.call(value)
}

pub fn proxy_typeof<'lua>(lua: &'lua Lua, value: LuaValue<'lua>) -> LuaResult<LuaString<'lua>> {
    if let LuaValue::UserData(u) = &value {
        if u.is::<TaskReference>() {
            return lua.create_string("thread");
        }
        #[cfg(feature = "roblox")]
        {
            if let Some(type_name) = u.roblox_type_name() {
                return lua.create_string(type_name);
            }
        }
    }
    lua.named_registry_value::<LuaFunction>("typeof")?
        .call(value)
}

// TODO: Add an override for tostring that formats errors in a nicer way
