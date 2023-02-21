use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    process::{ExitCode, Stdio},
};

use directories::UserDirs;
use mlua::prelude::*;
use os_str_bytes::RawOsString;
use tokio::process::Command;

use crate::lua::{
    process::pipe_and_inherit_child_process_stdio, table::TableBuilder, task::TaskScheduler,
};

const PROCESS_EXIT_IMPL_LUA: &str = r#"
exit(...)
yield()
"#;

pub fn create(lua: &'static Lua, args_vec: Vec<String>) -> LuaResult<LuaTable> {
    let cwd_str = {
        let cwd = env::current_dir()?.canonicalize()?;
        let cwd_str = cwd.to_string_lossy().to_string();
        if !cwd_str.ends_with('/') {
            format!("{cwd_str}/")
        } else {
            cwd_str
        }
    };
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
    // Create our process exit function, this is a bit involved since
    // we have no way to yield from c / rust, we need to load a lua
    // chunk that will set the exit code and yield for us instead
    let process_exit_env_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let process_exit_env_exit: LuaFunction = lua.create_function(|lua, code: Option<u8>| {
        let exit_code = code.map_or(ExitCode::SUCCESS, ExitCode::from);
        let sched = lua
            .app_data_ref::<&TaskScheduler>()
            .expect("Missing task scheduler - make sure it is added as a lua app data before the first scheduler resumption");
        sched.set_exit_code(exit_code);
        Ok(())
    })?;
    let process_exit = lua
        .load(PROCESS_EXIT_IMPL_LUA)
        .set_name("=process.exit")?
        .set_environment(
            TableBuilder::new(lua)?
                .with_value("yield", process_exit_env_yield)?
                .with_value("exit", process_exit_env_exit)?
                .build_readonly()?,
        )?
        .into_function()?;
    // Create the full process table
    TableBuilder::new(lua)?
        .with_value("args", args_tab)?
        .with_value("cwd", cwd_str)?
        .with_value("env", env_tab)?
        .with_value("exit", process_exit)?
        .with_async_function("spawn", process_spawn)?
        .build_readonly()
}

fn process_env_get<'a>(
    lua: &'static Lua,
    (_, key): (LuaValue<'a>, String),
) -> LuaResult<LuaValue<'a>> {
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

fn process_env_set(
    _: &'static Lua,
    (_, key, value): (LuaValue, String, Option<String>),
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

async fn process_spawn<'a>(
    lua: &'static Lua,
    (mut program, args, options): (String, Option<Vec<String>>, Option<LuaTable<'a>>),
) -> LuaResult<LuaTable<'a>> {
    // Parse any given options or create defaults
    let (child_cwd, child_envs, child_shell, child_stdio_inherit) = match options {
        Some(options) => {
            let mut cwd = env::current_dir()?;
            let mut envs = HashMap::new();
            let mut shell = None;
            let mut inherit = false;
            match options.raw_get("cwd")? {
                LuaValue::Nil => {}
                LuaValue::String(s) => {
                    cwd = PathBuf::from(s.to_string_lossy().to_string());
                    // Substitute leading tilde (~) for the actual home dir
                    if cwd.starts_with("~") {
                        if let Some(user_dirs) = UserDirs::new() {
                            cwd = user_dirs.home_dir().join(cwd.strip_prefix("~").unwrap())
                        }
                    };
                    if !cwd.exists() {
                        return Err(LuaError::RuntimeError(
                            "Invalid value for option 'cwd' - path does not exist".to_string(),
                        ));
                    }
                }
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "Invalid type for option 'cwd' - expected 'string', got '{}'",
                        value.type_name()
                    )))
                }
            }
            match options.raw_get("env")? {
                LuaValue::Nil => {}
                LuaValue::Table(t) => {
                    for pair in t.pairs::<String, String>() {
                        let (k, v) = pair?;
                        envs.insert(k, v);
                    }
                }
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "Invalid type for option 'env' - expected 'table', got '{}'",
                        value.type_name()
                    )))
                }
            }
            match options.raw_get("shell")? {
                LuaValue::Nil => {}
                LuaValue::String(s) => shell = Some(s.to_string_lossy().to_string()),
                LuaValue::Boolean(true) => {
                    shell = match env::consts::FAMILY {
                        "unix" => Some("/bin/sh".to_string()),
                        "windows" => Some("/bin/sh".to_string()),
                        _ => None,
                    };
                }
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "Invalid type for option 'shell' - expected 'true' or 'string', got '{}'",
                        value.type_name()
                    )))
                }
            }
            match options.raw_get("stdio")? {
                LuaValue::Nil => {}
                LuaValue::String(s) => {
                    match s.to_str()? {
                        "inherit" => {
                            inherit = true;
                        },
                        "default" => {
                            inherit = false;
                        }
                        _ => return Err(LuaError::RuntimeError(
                            format!("Invalid value for option 'stdio' - expected 'inherit' or 'default', got '{}'", s.to_string_lossy()),
                        ))
                    }
                }
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "Invalid type for option 'stdio' - expected 'string', got '{}'",
                        value.type_name()
                    )))
                }
            }
            Ok::<_, LuaError>((cwd, envs, shell, inherit))
        }
        None => Ok((env::current_dir()?, HashMap::new(), None, false)),
    }?;
    // Run a shell using the command param if wanted
    let child_args = if let Some(shell) = child_shell {
        let shell_args = match args {
            Some(args) => vec!["-c".to_string(), format!("{} {}", program, args.join(" "))],
            None => vec!["-c".to_string(), program],
        };
        program = shell;
        Some(shell_args)
    } else {
        args
    };
    // Create command with the wanted options
    let mut cmd = match child_args {
        None => Command::new(program),
        Some(args) => {
            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd
        }
    };
    // Set dir to run in and env variables
    cmd.current_dir(child_cwd);
    cmd.envs(child_envs);
    // Spawn the child process
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Inherit the output and stderr if wanted
    let result = if child_stdio_inherit {
        pipe_and_inherit_child_process_stdio(child).await
    } else {
        let output = child.wait_with_output().await?;
        Ok((output.status, output.stdout, output.stderr))
    };
    // Extract result
    let (status, stdout, stderr) = result?;
    // NOTE: If an exit code was not given by the child process,
    // we default to 1 if it yielded any error output, otherwise 0
    let code = status.code().unwrap_or(match stderr.is_empty() {
        true => 0,
        false => 1,
    });
    // Construct and return a readonly lua table with results
    TableBuilder::new(lua)?
        .with_value("ok", code == 0)?
        .with_value("code", code)?
        .with_value("stdout", lua.create_string(&stdout)?)?
        .with_value("stderr", lua.create_string(&stderr)?)?
        .build_readonly()
}
