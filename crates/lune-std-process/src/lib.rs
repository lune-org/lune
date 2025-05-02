#![allow(clippy::cargo_common_metadata)]

use std::{
    env::consts::{ARCH, OS},
    path::MAIN_SEPARATOR,
    process::Stdio,
};

use mlua::prelude::*;
use mlua_luau_scheduler::Functions;

use lune_utils::{
    path::get_current_dir,
    process::{ProcessArgs, ProcessEnv},
    TableBuilder,
};

mod create;
mod exec;
mod options;

use self::options::ProcessSpawnOptions;

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `process` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

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

    // Extract stored userdatas for args + env, the runtime struct should always provide this
    let process_args = lua
        .app_data_ref::<ProcessArgs>()
        .ok_or_else(|| LuaError::runtime("Missing process args in Lua app data"))?
        .clone();
    let process_env = lua
        .app_data_ref::<ProcessEnv>()
        .ok_or_else(|| LuaError::runtime("Missing process env in Lua app data"))?
        .clone();

    // Create our process exit function, the scheduler crate provides this
    let fns = Functions::new(lua.clone())?;
    let process_exit = fns.exit;

    // Create the full process table
    TableBuilder::new(lua)?
        .with_value("os", os)?
        .with_value("arch", arch)?
        .with_value("endianness", endianness)?
        .with_value("args", process_args)?
        .with_value("cwd", cwd_str)?
        .with_value("env", process_env)?
        .with_value("exit", process_exit)?
        .with_async_function("exec", process_exec)?
        .with_function("create", process_create)?
        .build_readonly()
}

async fn process_exec(
    lua: Lua,
    (program, args, mut options): (String, ProcessArgs, ProcessSpawnOptions),
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
    (program, args, options): (String, ProcessArgs, ProcessSpawnOptions),
) -> LuaResult<LuaValue> {
    let child = options
        .into_command(program, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    create::Child::new(lua, child).into_lua(lua)
}
