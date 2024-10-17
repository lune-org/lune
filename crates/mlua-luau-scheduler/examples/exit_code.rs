#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cargo_common_metadata)]

use async_io::block_on;

use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

const MAIN_SCRIPT: &str = include_str!("./lua/exit_code.luau");

pub fn main() -> LuaResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    // Set up persistent Lua environment
    let lua = Lua::new();
    let sched = Scheduler::new(&lua);
    let fns = Functions::new(&lua)?;

    lua.globals().set("exit", fns.exit)?;

    // Load the main script into the scheduler
    let main = lua.load(MAIN_SCRIPT);
    sched.push_thread_front(main, ())?;

    // Run until completion
    block_on(sched.run());

    // Verify that we got a correct exit code
    let code = sched.get_exit_code().unwrap_or_default();
    assert_eq!(code, 1);

    Ok(())
}

#[test]
fn test_exit_code() -> LuaResult<()> {
    main()
}
