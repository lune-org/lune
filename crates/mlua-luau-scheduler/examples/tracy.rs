/*
    NOTE: This example is the same as "lots_of_threads", but with tracy set up for performance profiling.

    How to run:

    1. Install tracy
       - Follow the instructions at https://github.com/wolfpld/tracy
       - Or install via something like homebrew: `brew install tracy`
    2. Run the server (`tracy`) in a terminal
    3. Run the example in another terminal
       - `export RUST_LOG=trace`
       - `cargo run --example tracy`
*/

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cargo_common_metadata)]

use std::time::Duration;

use async_io::{Timer, block_on};
use tracing_subscriber::layer::SubscriberExt;
use tracing_tracy::{TracyLayer, client::Client as TracyClient};

use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

const MAIN_SCRIPT: &str = include_str!("./lua/lots_of_threads.luau");

const ONE_NANOSECOND: Duration = Duration::from_nanos(1);

pub fn main() -> LuaResult<()> {
    let _client = TracyClient::start();
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(TracyLayer::default()),
    );

    // Set up persistent Lua environment
    let lua = Lua::new();
    let sched = Scheduler::new(lua.clone());
    let fns = Functions::new(lua.clone())?;

    lua.globals().set("spawn", fns.spawn)?;
    lua.globals().set(
        "sleep",
        lua.create_async_function(|_, ()| async move {
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
