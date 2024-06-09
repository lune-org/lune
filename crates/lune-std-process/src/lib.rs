#![allow(clippy::cargo_common_metadata)]

use std::{
    cell::RefCell,
    env::{
        self,
        consts::{ARCH, OS},
    },
    path::MAIN_SEPARATOR,
    process::Stdio,
    rc::Rc,
};

use mlua::prelude::*;

use lune_utils::TableBuilder;
use mlua_luau_scheduler::{Functions, LuaSpawnExt};
use options::ProcessSpawnOptionsStdio;
use os_str_bytes::RawOsString;
use stream::{ChildProcessReader, ChildProcessWriter};
use tokio::{io::AsyncWriteExt, process::Child};

mod options;
mod stream;
mod tee_writer;
mod wait_for_child;

use self::options::ProcessSpawnOptions;
use self::wait_for_child::wait_for_child;

use lune_utils::path::get_current_dir;

/**
    Creates the `process` standard library module.

    # Errors

    Errors when out of memory.
*/
#[allow(clippy::missing_panics_doc)]
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let mut cwd_str = get_current_dir()
        .to_str()
        .expect("cwd should be valid UTF-8")
        .to_string();
    if !cwd_str.ends_with(MAIN_SEPARATOR) {
        cwd_str.push(MAIN_SEPARATOR);
    }
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
        .with_async_function("exec", process_exec)?
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

async fn process_exec(
    lua: &Lua,
    (program, args, options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaTable> {
    let res = lua
        .spawn(async move {
            let cmd = spawn_command(program, args, options.clone()).await?;
            wait_for_child(cmd, options.stdio.stdout, options.stdio.stderr).await
        })
        .await?;

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

#[allow(clippy::await_holding_refcell_ref)]
async fn process_spawn(
    lua: &Lua,
    (program, args, options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaTable> {
    // Spawn does not accept stdio options, so we remove them from the options
    // and use the defaults instead
    let mut spawn_options = options.clone();
    spawn_options.stdio = ProcessSpawnOptionsStdio::default();

    let (stdin_tx, stdin_rx) = tokio::sync::oneshot::channel();
    let (stdout_tx, stdout_rx) = tokio::sync::oneshot::channel();
    let (stderr_tx, stderr_rx) = tokio::sync::oneshot::channel();
    let (code_tx, code_rx) = tokio::sync::broadcast::channel(4);
    let code_rx_rc = Rc::new(RefCell::new(code_rx));

    tokio::spawn(async move {
        let mut child = spawn_command(program, args, spawn_options)
            .await
            .expect("Could not spawn child process");
        stdin_tx
            .send(child.stdin.take())
            .expect("Stdin receiver was unexpectedly dropped");
        stdout_tx
            .send(child.stdout.take())
            .expect("Stdout receiver was unexpectedly dropped");
        stderr_tx
            .send(child.stderr.take())
            .expect("Stderr receiver was unexpectedly dropped");

        let res = child
            .wait_with_output()
            .await
            .expect("Failed to get status and output of spawned child process");

        let code = res
            .status
            .code()
            .unwrap_or(i32::from(!res.stderr.is_empty()));

        code_tx
            .send(code)
            .expect("ExitCode receiver was unexpectedly dropped");
    });

    // TODO: Remove the lua errors since we no longer accept stdio options for spawn
    TableBuilder::new(lua)?
        .with_value(
            "stdout",
            ChildProcessReader(
                stdout_rx
                    .await
                    .expect("Stdout sender unexpectedly dropped")
                    .ok_or(LuaError::runtime(
                        "Cannot read from stdout when it is not piped",
                    ))?,
            ),
        )?
        .with_value(
            "stderr",
            ChildProcessReader(
                stderr_rx
                    .await
                    .expect("Stderr sender unexpectedly dropped")
                    .ok_or(LuaError::runtime(
                        "Cannot read from stderr when it is not piped",
                    ))?,
            ),
        )?
        .with_value(
            "stdin",
            ChildProcessWriter(
                stdin_rx
                    .await
                    .expect("Stdin sender unexpectedly dropped")
                    .unwrap(),
            ),
        )?
        .with_async_function("status", move |lua, ()| {
            let code_rx_rc_clone = Rc::clone(&code_rx_rc);
            async move {
                let code = code_rx_rc_clone
                    .borrow_mut()
                    .recv()
                    .await
                    .expect("Code sender unexpectedly dropped");

                TableBuilder::new(lua)?
                    .with_value("code", code)?
                    .with_value("ok", code == 0)?
                    .build_readonly()
            }
        })?
        .build_readonly()
}

async fn spawn_command(
    program: String,
    args: Option<Vec<String>>,
    mut options: ProcessSpawnOptions,
) -> LuaResult<Child> {
    let stdout = options.stdio.stdout;
    let stderr = options.stdio.stderr;
    let stdin = options.stdio.stdin.take();

    // TODO: Have an stdin_kind which the user can supply as piped or not
    // TODO: Maybe even revamp the stdout/stderr kinds? User should only use
    // piped when they are sure they want to read the stdout. Currently we default
    // to piped
    let mut child = options
        .into_command(program, args)
        .stdin(Stdio::piped())
        .stdout(stdout.as_stdio())
        .stderr(stderr.as_stdio())
        .spawn()?;

    if let Some(stdin) = stdin {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(&stdin).await.into_lua_err()?;
    }

    Ok(child)
}
