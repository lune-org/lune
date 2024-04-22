#![allow(clippy::cargo_common_metadata)]

use std::{
    env::{
        self,
        consts::{ARCH, OS},
    },
    process::Stdio,
};

use mlua::prelude::*;

use lune_utils::TableBuilder;
use mlua_luau_scheduler::{Functions, LuaSpawnExt};
use os_str_bytes::RawOsString;
use tokio::io::AsyncWriteExt;

mod options;
mod tee_writer;
mod wait_for_child;

use self::options::ProcessSpawnOptions;
use self::wait_for_child::{wait_for_child, WaitForChildResult};

use lune_utils::path::get_current_dir;

/**
    Creates the `process` standard library module.

    # Errors

    Errors when out of memory.
*/
#[allow(clippy::missing_panics_doc)]
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let cwd_str = get_current_dir()
        .to_str()
        .expect("cwd should be valid UTF-8")
        .to_string();
    // Create constants for OS & processor architecture
    let os = lua.create_string(&OS.to_lowercase())?;
    let arch = lua.create_string(&ARCH.to_lowercase())?;
    // Create readonly args array
    let args_vec = lua
        .app_data_ref::<Vec<String>>()
        .ok_or_else(|| LuaError::runtime("Missing args vec in Lua app data"))?
        .clone();
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
    // Create our process exit function, the scheduler crate provides this
    let fns = Functions::new(lua)?;
    let process_exit = fns.exit;
    // Create the full process table
    TableBuilder::new(lua)?
        .with_value("os", os)?
        .with_value("arch", arch)?
        .with_value("args", args_tab)?
        .with_value("cwd", cwd_str)?
        .with_value("env", env_tab)?
        .with_value("exit", process_exit)?
        .with_async_function("spawn", process_spawn)?
        .build_readonly()
}

fn process_env_get<'lua>(
    lua: &'lua Lua,
    (_, key): (LuaValue<'lua>, String),
) -> LuaResult<LuaValue<'lua>> {
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

fn process_env_set<'lua>(
    _: &'lua Lua,
    (_, key, value): (LuaValue<'lua>, String, Option<String>),
) -> LuaResult<()> {
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

fn process_env_iter<'lua>(
    lua: &'lua Lua,
    (_, ()): (LuaValue<'lua>, ()),
) -> LuaResult<LuaFunction<'lua>> {
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

async fn process_spawn(
    lua: &Lua,
    (program, args, options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaTable> {
    let res = lua
        .spawn(spawn_command(program, args, options))
        .await
        .expect("Failed to receive result of spawned process");

    /*
        NOTE: If an exit code was not given by the child process,
        we default to 1 if it yielded any error output, otherwise 0

        An exit code may be missing if the process was terminated by
        some external signal, which is the only time we use this default
    */
    let code = res
        .status
        .code()
        .unwrap_or(i32::from(!res.stderr.is_empty()));

    // Construct and return a readonly lua table with results
    TableBuilder::new(lua)?
        .with_value("ok", code == 0)?
        .with_value("code", code)?
        .with_value("stdout", lua.create_string(&res.stdout)?)?
        .with_value("stderr", lua.create_string(&res.stderr)?)?
        .build_readonly()
}

async fn spawn_command(
    program: String,
    args: Option<Vec<String>>,
    mut options: ProcessSpawnOptions,
) -> LuaResult<WaitForChildResult> {
    let stdout = options.stdio.stdout;
    let stderr = options.stdio.stderr;
    let stdin = options.stdio.stdin.take();

    let mut child = options
        .into_command(program, args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(stdout.as_stdio())
        .stderr(stderr.as_stdio())
        .spawn()?;

    if let Some(stdin) = stdin {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(&stdin).await.into_lua_err()?;
    }

    wait_for_child(child, stdout, stderr).await
}
