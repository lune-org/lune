use std::{
    env::{self, consts},
    path,
    process::{ExitStatus, Stdio},
};

use dunce::canonicalize;
use mlua::prelude::*;
use os_str_bytes::RawOsString;
use tokio::io::AsyncWriteExt;

use crate::lune::{scheduler::Scheduler, util::TableBuilder};

mod tee_writer;

mod pipe_inherit;
use pipe_inherit::pipe_and_inherit_child_process_stdio;

mod options;
use options::ProcessSpawnOptions;

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

async fn process_spawn(
    lua: &Lua,
    (program, args, options): (String, Option<Vec<String>>, ProcessSpawnOptions),
) -> LuaResult<LuaTable> {
    /*
        Spawn the new process in the background, letting the tokio
        runtime place it on a different thread if possible / necessary

        Note that we have to use our scheduler here, we can't
        use anything like tokio::task::spawn because our lua
        scheduler will not drive those futures to completion
    */
    let sched = lua
        .app_data_ref::<&Scheduler>()
        .expect("Lua struct is missing scheduler");

    let (status, stdout, stderr) = sched
        .spawn(spawn_command(program, args, options))
        .await
        .expect("Failed to receive result of spawned process")?;

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

async fn spawn_command(
    program: String,
    args: Option<Vec<String>>,
    options: ProcessSpawnOptions,
) -> LuaResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let inherit_stdio = options.inherit_stdio;
    let stdin = options.stdin.clone();

    let mut child = options
        .into_command(program, args)
        .stdin(match stdin.is_some() {
            true => Stdio::piped(),
            false => Stdio::null(),
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // If the stdin option was provided, we write that to the child
    if let Some(stdin) = stdin {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin
            .write_all(stdin.as_bytes())
            .await
            .into_lua_err()?;
    }

    if inherit_stdio {
        pipe_and_inherit_child_process_stdio(child).await
    } else {
        let output = child.wait_with_output().await?;
        Ok((output.status, output.stdout, output.stderr))
    }
}
