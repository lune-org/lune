#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cargo_common_metadata)]

use std::time::{Duration, Instant};

use async_io::{block_on, Timer};

use mlua::prelude::*;
use mlua_luau_scheduler::Scheduler;

const MAIN_SCRIPT: &str = include_str!("./lua/basic_sleep.luau");

pub fn main() -> LuaResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    // Set up persistent Lua environment
    let lua = Lua::new();
    lua.globals().set(
        "sleep",
        lua.create_async_function(|_, duration: f64| async move {
            let before = Instant::now();
            let after = Timer::after(Duration::from_secs_f64(duration)).await;
            Ok((after - before).as_secs_f64())
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
fn test_basic_sleep() -> LuaResult<()> {
    main()
}
