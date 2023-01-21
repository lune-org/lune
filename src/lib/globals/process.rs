use std::{
    env,
    process::{exit, Stdio},
};

use mlua::{
    Error, Function, Lua, MetaMethod, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value,
};
use os_str_bytes::RawOsString;
use tokio::process::Command;

pub struct Process {
    args: Vec<String>,
}

impl Default for Process {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl Process {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl UserData for Process {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("args", |lua, this| {
            // TODO: Use the same strategy as env uses below to avoid
            // copying each time args are accessed? is it worth it?
            let tab = lua.create_table()?;
            for arg in &this.args {
                tab.push(arg.clone())?;
            }
            tab.set_readonly(true);
            Ok(tab)
        });
        fields.add_field_method_get("env", |lua, _| {
            let meta = lua.create_table()?;
            meta.raw_set(
                MetaMethod::Index.name(),
                lua.create_function(process_env_get)?,
            )?;
            meta.raw_set(
                MetaMethod::NewIndex.name(),
                lua.create_function(process_env_set)?,
            )?;
            meta.raw_set(
                MetaMethod::Iter.name(),
                lua.create_function(process_env_iter)?,
            )?;
            let tab = lua.create_table()?;
            tab.set_metatable(Some(meta));
            tab.set_readonly(true);
            Ok(tab)
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("exit", process_exit);
        methods.add_async_function("spawn", process_spawn);
    }
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
