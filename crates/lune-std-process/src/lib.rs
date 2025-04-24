#![allow(clippy::cargo_common_metadata)]

use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    path::MAIN_SEPARATOR,
    process::Stdio,
};

use mlua::prelude::*;
use mlua_luau_scheduler::Functions;

use os_str_bytes::RawOsString;

use lune_utils::{path::get_current_dir, TableBuilder};

mod create;
mod exec;
mod options;

use self::options::ProcessSpawnOptions;

/**
    Creates the `process` standard library module.

    # Errors

    Errors when out of memory.
*/
#[allow(clippy::missing_panics_doc)]
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    let mut cwd_str = get_current_dir()
        .to_str()
        .expect("cwd should be valid UTF-8")
        .to_string();
    if !cwd_str.ends_with(MAIN_SEPARATOR) {
        cwd_str.push(MAIN_SEPARATOR);
    }

    // Create constants for OS & processor architecture
    let os = lua.create_string(OS.to_lowercase())?;
    let arch = lua.create_string(ARCH.to_lowercase())?;
    let endianness = lua.create_string(if cfg!(target_endian = "big") {
        "big"
    } else {
        "little"
    })?;

    // Create readonly args array
    let args_vec = lua
        .app_data_ref::<Vec<String>>()
        .ok_or_else(|| LuaError::runtime("Missing args vec in Lua app data"))?
        .clone();
    let args_tab = TableBuilder::new(lua.clone())?
        .with_sequential_values(args_vec)?
        .build_readonly()?;

    // Create proxied table for env that gets & sets real env vars
    let env_tab = TableBuilder::new(lua.clone())?
        .with_metatable(
            TableBuilder::new(lua.clone())?
                .with_function(LuaMetaMethod::Index.name(), process_env_get)?
                .with_function(LuaMetaMethod::NewIndex.name(), process_env_set)?
                .with_function(LuaMetaMethod::Iter.name(), process_env_iter)?
                .build_readonly()?,
        )?
        .build_readonly()?;

    // Create our process exit function, the scheduler crate provides this
    let fns = Functions::new(lua.clone())?;
    let process_exit = fns.exit;

    // Create the full process table
    TableBuilder::new(lua)?
        .with_value("os", os)?
        .with_value("arch", arch)?
        .with_value("endianness", endianness)?
        .with_value("args", args_tab)?
        .with_value("cwd", cwd_str)?
        .with_value("env", env_tab)?
        .with_value("exit", process_exit)?
        .with_async_function("exec", process_exec)?
        .with_function("create", process_create)?
        .build_readonly()
}

fn process_env_get(lua: &Lua, (_, key): (LuaValue, String)) -> LuaResult<LuaValue> {
    match env::var_os(key) {
        Some(value) => {
            let raw_value = RawOsString::new(value);
            Ok(LuaValue::String(
                lua.create_string(raw_value.to_raw_bytes())?,
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
    } else if let Some(value) = value {
        // Make sure value is valid, otherwise set_var will panic
        if value.contains('\0') {
            Err(LuaError::RuntimeError(
                "Value must not contain the NUL character".to_string(),
            ))
        } else {
            env::set_var(&key, &value);
            Ok(())
        }
    } else {
        env::remove_var(&key);
        Ok(())
    }
}

fn process_env_iter(lua: &Lua, (_, ()): (LuaValue, ())) -> LuaResult<LuaFunction> {
    let mut vars = env::vars_os().collect::<Vec<_>>().into_iter();
    lua.create_function_mut(move |lua, (): ()| match vars.next() {
        Some((key, value)) => {
            let raw_key = RawOsString::new(key);
            let raw_value = RawOsString::new(value);
            Ok((
                LuaValue::String(lua.create_string(raw_key.to_raw_bytes())?),
                LuaValue::String(lua.create_string(raw_value.to_raw_bytes())?),
            ))
        }
        None => Ok((LuaValue::Nil, LuaValue::Nil)),
    })
}

async fn process_exec(
    lua: Lua,
    (program, args, mut options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaTable> {
    let stdin = options.stdio.stdin.take();
    let stdout = options.stdio.stdout;
    let stderr = options.stdio.stderr;

    let child = options
        .into_command(program, args)
        .stdin(Stdio::piped())
        .stdout(stdout.as_stdio())
        .stderr(stderr.as_stdio())
        .spawn()?;

    exec::exec(lua, child, stdin, stdout, stderr).await
}

fn process_create(
    lua: &Lua,
    (program, args, options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaValue> {
    let child = options
        .into_command(program, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    create::Child::new(lua, child).into_lua(lua)
}
