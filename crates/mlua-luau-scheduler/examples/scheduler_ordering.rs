#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cargo_common_metadata)]

use std::time::{Duration, Instant};

use async_io::{block_on, Timer};

use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

const MAIN_SCRIPT: &str = include_str!("./lua/scheduler_ordering.luau");

pub fn main() -> LuaResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    // Set up persistent Lua environment
    let lua = Lua::new();
    let sched = Scheduler::new(lua.clone());
    let fns = Functions::new(lua.clone())?;

    lua.globals().set("spawn", fns.spawn)?;
    lua.globals().set("defer", fns.defer)?;
    lua.globals().set(
        "sleep",
        lua.create_async_function(|_, duration: Option<f64>| async move {
            let duration = duration.unwrap_or_default().max(1.0 / 250.0);
            let before = Instant::now();
            let after = Timer::after(Duration::from_secs_f64(duration)).await;
            Ok((after - before).as_secs_f64())
        })?,
    )?;

    // Load the main script into the scheduler, and keep track of the thread we spawn
    let main = lua.load(MAIN_SCRIPT);
    let id = sched.push_thread_front(main, ())?;

    // Run until completion
    block_on(sched.run());

    // We should have gotten proper values back from our script
    let res = sched.get_thread_result(id).unwrap().unwrap();
    let nums = Vec::<usize>::from_lua_multi(res, &lua)?;
    assert_eq!(nums, vec![1, 2, 3, 4, 5, 6]);

    Ok(())
}

#[test]
fn test_scheduler_ordering() -> LuaResult<()> {
    main()
}
