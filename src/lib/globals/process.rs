use std::{
    env,
    process::{exit, Stdio},
};

use mlua::{Error, Function, Lua, MetaMethod, Result, Table, Value};
use os_str_bytes::RawOsString;
use tokio::process::Command;

pub fn new(lua: &Lua, args_vec: Vec<String>) -> Result<Table> {
    // Create readonly args array
    let inner_args = lua.create_table()?;
    for arg in &args_vec {
        inner_args.push(arg.clone())?;
    }
    inner_args.set_readonly(true);
    // Create proxied env metatable that gets & sets real env vars
    let inner_env_meta = lua.create_table()?;
    inner_env_meta.raw_set(
        MetaMethod::Index.name(),
        lua.create_function(process_env_get)?,
    )?;
    inner_env_meta.raw_set(
        MetaMethod::NewIndex.name(),
        lua.create_function(process_env_set)?,
    )?;
    inner_env_meta.raw_set(
        MetaMethod::Iter.name(),
        lua.create_function(process_env_iter)?,
    )?;
    inner_env_meta.set_readonly(true);
    // Create blank table for env with the metatable
    let inner_env = lua.create_table()?;
    inner_env.set_metatable(Some(inner_env_meta));
    inner_env.set_readonly(true);
    // Create the full process table
    let tab = lua.create_table()?;
    tab.raw_set("args", inner_args)?;
    tab.raw_set("env", inner_env)?;
    tab.raw_set("exit", lua.create_function(process_exit)?)?;
    tab.raw_set("spawn", lua.create_async_function(process_spawn)?)?;
    tab.set_readonly(true);
    Ok(tab)
}

fn process_env_get<'lua>(lua: &'lua Lua, (_, key): (Value<'lua>, String)) -> Result<Value<'lua>> {
    match env::var_os(key) {
        Some(value) => {
            let raw_value = RawOsString::new(value);
            Ok(Value::String(lua.create_string(raw_value.as_raw_bytes())?))
        }
        None => Ok(Value::Nil),
    }
}

fn process_env_set(_: &Lua, (_, key, value): (Value, String, Option<String>)) -> Result<()> {
    // Make sure key is valid, otherwise set_var will panic
    if key.is_empty() {
        return Err(Error::RuntimeError("Key must not be empty".to_string()));
    } else if key.contains('=') {
        return Err(Error::RuntimeError(
            "Key must not contain the equals character '='".to_string(),
        ));
    } else if key.contains('\0') {
        return Err(Error::RuntimeError(
            "Key must not contain the NUL character".to_string(),
        ));
    }
    match value {
        Some(value) => {
            // Make sure value is valid, otherwise set_var will panic
            if value.contains('\0') {
                return Err(Error::RuntimeError(
                    "Value must not contain the NUL character".to_string(),
                ));
            }
            env::set_var(&key, &value);
        }
        None => env::remove_var(&key),
    }
    Ok(())
}

fn process_env_iter<'lua>(lua: &'lua Lua, (_, _): (Value<'lua>, ())) -> Result<Function<'lua>> {
    let mut vars = env::vars_os();
    lua.create_function_mut(move |lua, _: ()| match vars.next() {
        Some((key, value)) => {
            let raw_key = RawOsString::new(key);
            let raw_value = RawOsString::new(value);
            Ok((
                Value::String(lua.create_string(raw_key.as_raw_bytes())?),
                Value::String(lua.create_string(raw_value.as_raw_bytes())?),
            ))
        }
        None => Ok((Value::Nil, Value::Nil)),
    })
}

fn process_exit(_: &Lua, exit_code: Option<i32>) -> Result<()> {
    // TODO: Exit gracefully to the root with an Ok
    // result instead of completely exiting the process
    if let Some(code) = exit_code {
        exit(code);
    } else {
        exit(0)
    }
}

async fn process_spawn(lua: &Lua, (program, args): (String, Option<Vec<String>>)) -> Result<Table> {
    // Create and spawn a child process, and
    // wait for it to terminate with output
    let mut cmd = Command::new(program);
    if let Some(args) = args {
        cmd.args(args);
    }
    let child = cmd
        .current_dir(env::current_dir().map_err(mlua::Error::external)?)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(mlua::Error::external)?;
    let output = child
        .wait_with_output()
        .await
        .map_err(mlua::Error::external)?;
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
    let table = lua.create_table()?;
    table.raw_set("ok", code == 0)?;
    table.raw_set("code", code)?;
    table.raw_set("stdout", lua.create_string(&output.stdout)?)?;
    table.raw_set("stderr", lua.create_string(&output.stderr)?)?;
    table.set_readonly(true);
    Ok(table)
}
