use std::{
    env,
    process::{Command, Stdio},
    sync::Weak,
};

use mlua::prelude::*;
use os_str_bytes::RawOsString;
use smol::{channel::Sender, unblock};

use crate::{utils::table::TableBuilder, LuneMessage};

pub fn create(lua: &Lua, args_vec: Vec<String>) -> LuaResult<()> {
    // Create readonly args array
    let args_tab = TableBuilder::new(lua)?
        .with_sequential_values(args_vec)?
        .build_readonly()?;
    // Create proxied table for env that gets & sets real env vars
    let env_tab = TableBuilder::new(lua)?
        .with_metatable(
            TableBuilder::new(lua)?
                .with_function(LuaMetaMethod::Index.name(), process_env_get)?
                .with_function(LuaMetaMethod::NewIndex.name(), process_env_set)?
                .with_function(LuaMetaMethod::Iter.name(), process_env_iter)?
                .build_readonly()?,
        )?
        .build_readonly()?;
    // Create the full process table
    lua.globals().raw_set(
        "process",
        TableBuilder::new(lua)?
            .with_value("args", args_tab)?
            .with_value("env", env_tab)?
            .with_async_function("exit", process_exit)?
            .with_async_function("spawn", process_spawn)?
            .build_readonly()?,
    )
}

fn process_env_get<'lua>(
    lua: &'lua Lua,
    (_, key): (LuaValue<'lua>, String),
) -> LuaResult<LuaValue<'lua>> {
    match env::var_os(key) {
        Some(value) => {
            let raw_value = RawOsString::new(value);
            Ok(LuaValue::String(
                lua.create_string(raw_value.as_raw_bytes())?,
            ))
        }
        None => Ok(LuaValue::Nil),
    }
}

fn process_env_set(_: &Lua, (_, key, value): (LuaValue, String, Option<String>)) -> LuaResult<()> {
    // Make sure key is valid, otherwise set_var will panic
    if key.is_empty() {
        Err(LuaError::RuntimeError("Key must not be empty".to_string()))
    } else if key.contains('=') {
        Err(LuaError::RuntimeError(
            "Key must not contain the equals character '='".to_string(),
        ))
    } else if key.contains('\0') {
        Err(LuaError::RuntimeError(
            "Key must not contain the NUL character".to_string(),
        ))
    } else {
        match value {
            Some(value) => {
                // Make sure value is valid, otherwise set_var will panic
                if value.contains('\0') {
                    Err(LuaError::RuntimeError(
                        "Value must not contain the NUL character".to_string(),
                    ))
                } else {
                    env::set_var(&key, &value);
                    Ok(())
                }
            }
            None => {
                env::remove_var(&key);
                Ok(())
            }
        }
    }
}

fn process_env_iter<'lua>(
    lua: &'lua Lua,
    (_, _): (LuaValue<'lua>, ()),
) -> LuaResult<LuaFunction<'lua>> {
    let mut vars = env::vars_os();
    lua.create_function_mut(move |lua, _: ()| match vars.next() {
        Some((key, value)) => {
            let raw_key = RawOsString::new(key);
            let raw_value = RawOsString::new(value);
            Ok((
                LuaValue::String(lua.create_string(raw_key.as_raw_bytes())?),
                LuaValue::String(lua.create_string(raw_value.as_raw_bytes())?),
            ))
        }
        None => Ok((LuaValue::Nil, LuaValue::Nil)),
    })
}

async fn process_exit(lua: &Lua, exit_code: Option<u8>) -> LuaResult<()> {
    let sender = lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    sender
        .send(LuneMessage::Exit(exit_code.unwrap_or(0)))
        .await
        .map_err(LuaError::external)?;
    Ok(())
}

async fn process_spawn(
    lua: &Lua,
    (program, args): (String, Option<Vec<String>>),
) -> LuaResult<LuaTable> {
    // Create and spawn a **blocking** child process to prevent
    // issues with yielding across the metamethod/c-call boundary
    let pwd = env::current_dir()?;
    let output = unblock(move || {
        let mut cmd = Command::new(program);
        if let Some(args) = args {
            cmd.args(args);
        }
        let child = cmd
            .current_dir(pwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        child.wait_with_output()
    })
    .await
    .map_err(LuaError::external)?;
    // NOTE: If an exit code was not given by the child process,
    // we default to 1 if it yielded any error output, otherwise 0
    let code = output
        .status
        .code()
        .unwrap_or(match output.stderr.is_empty() {
            true => 0,
            false => 1,
        });
    // Construct and return a readonly lua table with results
    TableBuilder::new(lua)?
        .with_value("ok", code == 0)?
        .with_value("code", code)?
        .with_value("stdout", lua.create_string(&output.stdout)?)?
        .with_value("stderr", lua.create_string(&output.stderr)?)?
        .build_readonly()
}
