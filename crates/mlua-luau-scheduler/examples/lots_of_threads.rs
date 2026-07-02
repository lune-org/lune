#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cargo_common_metadata)]

use std::time::Duration;

use async_io::{Timer, block_on};
use futures_lite::future::yield_now;

use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

const MAIN_SCRIPT: &str = include_str!("./lua/lots_of_threads.luau");

const ONE_NANOSECOND: Duration = Duration::from_nanos(1);

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
    lua.globals().set(
        "sleep",
        lua.create_async_function(|_, ()| async move {
            // Guarantee that the coroutine that calls this sleep function
            // always yields, even if the timer completes without doing so
            yield_now().await;
            // Obviously we can't sleep for a single nanosecond since
            // this uses OS scheduling under the hood, but we can try
            Timer::after(ONE_NANOSECOND).await;
            Ok(())
        })?,
    )?;

    // Load the main script into the scheduler
    let main = lua.load(MAIN_SCRIPT);
    sched.push_thread_front(main, ())?;

    // Run until completion
    block_on(sched.run());

    Ok(())
}

#[test]
fn test_lots_of_threads() -> LuaResult<()> {
    main()
}
