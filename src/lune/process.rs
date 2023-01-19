use std::{
    env::{self, VarError},
    process::{exit, Stdio},
};

use mlua::{Error, Lua, Result, Table, UserData, UserDataMethods, Value};
use tokio::process::Command;

pub struct LuneProcess();

impl LuneProcess {
    pub fn new() -> Self {
        Self()
    }
}

impl UserData for LuneProcess {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("getEnvVars", process_get_env_vars);
        methods.add_function("getEnvVar", process_get_env_var);
        methods.add_function("setEnvVar", process_set_env_var);
        methods.add_function("exit", process_exit);
        methods.add_async_function("spawn", process_spawn);
    }
}

fn process_get_env_vars(_: &Lua, _: ()) -> Result<Vec<String>> {
    let mut vars = Vec::new();
    for (key, _) in env::vars() {
        vars.push(key);
    }
    Ok(vars)
}

fn process_get_env_var(lua: &Lua, key: String) -> Result<Value> {
    match env::var(&key) {
        Ok(value) => Ok(Value::String(lua.create_string(&value)?)),
        Err(VarError::NotPresent) => Ok(Value::Nil),
        Err(VarError::NotUnicode(_)) => Err(Error::external(format!(
            "The env var '{}' contains invalid utf8",
            &key
        ))),
    }
}

fn process_set_env_var(_: &Lua, (key, value): (String, String)) -> Result<()> {
    Ok(env::set_var(&key, &value))
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
    // NOTE: Exit code defaults to 1 if it did not exist and if there
    // is any stderr, will otherwise default to 0 if it did not exist
    let code = output
        .status
        .code()
        .unwrap_or_else(|| match output.stderr.is_empty() {
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
