use std::time::Duration;

use mlua::prelude::*;

use mlua_luau_scheduler::Functions;
use tokio::time::{self, Instant};

use crate::lune::util::TableBuilder;

const DELAY_IMPL_LUA: &str = r#"
return defer(function(...)
    wait(select(1, ...))
    spawn(select(2, ...))
end, ...)
"#;

pub fn create(lua: &Lua) -> LuaResult<LuaTable<'_>> {
    let fns = Functions::new(lua)?;

    // Create wait & delay functions
    let task_wait = lua.create_async_function(wait)?;
    let task_delay_env = TableBuilder::new(lua)?
        .with_value("select", lua.globals().get::<_, LuaFunction>("select")?)?
        .with_value("spawn", fns.spawn.clone())?
        .with_value("defer", fns.defer.clone())?
        .with_value("wait", task_wait.clone())?
        .build_readonly()?;
    let task_delay = lua
        .load(DELAY_IMPL_LUA)
        .set_name("task.delay")
        .set_environment(task_delay_env)
        .into_function()?;

    // Overwrite resume & wrap functions on the coroutine global
    // with ones that are compatible with our scheduler
    let co = lua.globals().get::<_, LuaTable>("coroutine")?;
    co.set("resume", fns.resume.clone())?;
    co.set("wrap", fns.wrap.clone())?;

    TableBuilder::new(lua)?
        .with_value("cancel", fns.cancel)?
        .with_value("defer", fns.defer)?
        .with_value("delay", task_delay)?
        .with_value("spawn", fns.spawn)?
        .with_value("wait", task_wait)?
        .build_readonly()
}

async fn wait(_: &Lua, secs: Option<f64>) -> LuaResult<f64> {
    let duration = Duration::from_secs_f64(secs.unwrap_or_default());

    let before = Instant::now();
    time::sleep(duration).await;
    let after = Instant::now();

    Ok((after - before).as_secs_f64())
}
