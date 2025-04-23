#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cargo_common_metadata)]

use std::io::ErrorKind;

use async_fs::read_to_string;
use async_io::block_on;

use mlua::prelude::*;
use mlua_luau_scheduler::{LuaSpawnExt, Scheduler};

const MAIN_SCRIPT: &str = include_str!("./lua/basic_spawn.luau");

pub fn main() -> LuaResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    // Set up persistent Lua environment
    let lua = Lua::new();
    lua.globals().set(
        "readFile",
        lua.create_async_function(|lua, path: String| async move {
            // Spawn background task that does not take up resources on the Lua thread
            let task = lua.spawn(async move {
                match read_to_string(path).await {
                    Ok(s) => Ok(Some(s)),
                    Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(e),
                }
            });

            // Wait for it to complete
            let result = task.await.into_lua_err();

            // We can also spawn local tasks that do take up resources
            // on the Lua thread, but that do not have the Send bound
            if result.is_ok() {
                lua.spawn_local(async move {
                    println!("File read successfully!");
                });
            }

            result
        })?,
    )?;

    // Load the main script into a scheduler
    let sched = Scheduler::new(lua.clone());
    let main = lua.load(MAIN_SCRIPT);
    sched.push_thread_front(main, ())?;

    // Run until completion
    block_on(sched.run());

    Ok(())
}

#[test]
fn test_basic_spawn() -> LuaResult<()> {
    main()
}
