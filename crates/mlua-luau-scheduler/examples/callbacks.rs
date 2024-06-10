#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;
use mlua_luau_scheduler::Scheduler;

use async_io::block_on;

const MAIN_SCRIPT: &str = include_str!("./lua/callbacks.luau");

pub fn main() -> LuaResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    // Set up persistent Lua environment
    let lua = Lua::new();

    // Create a new scheduler with custom callbacks
    let sched = Scheduler::new(&lua);
    sched.set_error_callback(|e| {
        println!(
            "Captured error from Lua!\n{}\n{e}\n{}",
            "-".repeat(15),
            "-".repeat(15)
        );
    });

    // Load the main script into the scheduler, and keep track of the thread we spawn
    let main = lua.load(MAIN_SCRIPT);
    let id = sched.push_thread_front(main, ())?;

    // Run until completion
    block_on(sched.run());

    // We should have gotten the error back from our script
    assert!(sched.get_thread_result(id).unwrap().is_err());

    Ok(())
}

#[test]
fn test_callbacks() -> LuaResult<()> {
    main()
}
