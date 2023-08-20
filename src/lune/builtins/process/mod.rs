use std::{
    collections::HashMap,
    env::{self, consts},
    path::{self, PathBuf},
    process::Stdio,
};

use directories::UserDirs;
use dunce::canonicalize;
use mlua::prelude::*;
use os_str_bytes::RawOsString;
use tokio::process::Command;

use crate::lune::{scheduler::Scheduler, util::TableBuilder};

mod tee_writer;

mod pipe_inherit;
use pipe_inherit::pipe_and_inherit_child_process_stdio;

const PROCESS_EXIT_IMPL_LUA: &str = r#"
exit(...)
yield()
"#;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let cwd_str = {
        let cwd = canonicalize(env::current_dir()?)?;
        let cwd_str = cwd.to_string_lossy().to_string();
        if !cwd_str.ends_with(path::MAIN_SEPARATOR) {
            format!("{cwd_str}{}", path::MAIN_SEPARATOR)
        } else {
            cwd_str
        }
    };
    // Create constants for OS & processor architecture
    let os = lua.create_string(&consts::OS.to_lowercase())?;
    let arch = lua.create_string(&consts::ARCH.to_lowercase())?;
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
    // Create our process exit function, this is a bit involved since
    // we have no way to yield from c / rust, we need to load a lua
    // chunk that will set the exit code and yield for us instead
    let coroutine_yield = lua
        .globals()
        .get::<_, LuaTable>("coroutine")?
        .get::<_, LuaFunction>("yield")?;
    let set_scheduler_exit_code = lua.create_function(|lua, code: Option<u8>| {
        let sched = lua
            .app_data_ref::<&Scheduler>()
            .expect("Lua struct is missing scheduler");
        sched.set_exit_code(code.unwrap_or_default());
        Ok(())
    })?;
    let process_exit = lua
        .load(PROCESS_EXIT_IMPL_LUA)
        .set_name("=process.exit")
        .set_environment(
            TableBuilder::new(lua)?
                .with_value("yield", coroutine_yield)?
                .with_value("exit", set_scheduler_exit_code)?
                .build_readonly()?,
        )
        .into_function()?;
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
                lua.create_string(raw_value.as_raw_bytes())?,
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
    let mut vars = env::vars_os().collect::<Vec<_>>().into_iter();
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

async fn process_spawn<'lua>(
    lua: &'static Lua,
    (mut program, args, options): (String, Option<Vec<String>>, Option<LuaTable<'lua>>),
) -> LuaResult<LuaTable<'lua>> {
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
