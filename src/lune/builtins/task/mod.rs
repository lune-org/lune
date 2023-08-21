use std::time::Duration;

use mlua::prelude::*;

use tokio::time::{self, Instant};

use crate::lune::{scheduler::Scheduler, util::TableBuilder};

mod tof;
use tof::LuaThreadOrFunction;

/*
    The spawn function needs special treatment,
    we need to yield right away to allow the
    spawned task to run until first yield

    1. Schedule this current thread at the front
    2. Schedule given thread/function at the front,
       the previous schedule now comes right after
    3. Give control over to the scheduler, which will
       resume the above tasks in order when its ready
*/
const SPAWN_IMPL_LUA: &str = r#"
push(currentThread())
local thread = push(...)
yield()
return thread
"#;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable<'_>> {
    let coroutine_running = lua
        .globals()
        .get::<_, LuaTable>("coroutine")?
        .get::<_, LuaFunction>("running")?;
    let coroutine_yield = lua
        .globals()
        .get::<_, LuaTable>("coroutine")?
        .get::<_, LuaFunction>("yield")?;
    let push_front =
        lua.create_function(|lua, (tof, args): (LuaThreadOrFunction, LuaMultiValue)| {
            let thread = tof.into_thread(lua)?;
            let sched = lua
                .app_data_ref::<&Scheduler>()
                .expect("Lua struct is missing scheduler");
            sched.push_front(thread.clone(), args)?;
            Ok(thread)
        })?;
    let task_spawn_env = TableBuilder::new(lua)?
        .with_value("currentThread", coroutine_running)?
        .with_value("yield", coroutine_yield)?
        .with_value("push", push_front)?
        .build_readonly()?;
    let task_spawn = lua
        .load(SPAWN_IMPL_LUA)
        .set_name("task.spawn")
        .set_environment(task_spawn_env)
        .into_function()?;

    TableBuilder::new(lua)?
        .with_function("cancel", task_cancel)?
        .with_function("defer", task_defer)?
        .with_function("delay", task_delay)?
        .with_value("spawn", task_spawn)?
        .with_async_function("wait", task_wait)?
        .build_readonly()
}

fn task_cancel(lua: &Lua, thread: LuaThread) -> LuaResult<()> {
    let close = lua
        .globals()
        .get::<_, LuaTable>("coroutine")?
        .get::<_, LuaFunction>("close")?;
    match close.call(thread) {
        Err(LuaError::CoroutineInactive) => Ok(()),
        Err(e) => Err(e),
        Ok(()) => Ok(()),
    }
}

fn task_defer<'lua>(
    lua: &'lua Lua,
    (tof, args): (LuaThreadOrFunction<'lua>, LuaMultiValue<'_>),
) -> LuaResult<LuaThread<'lua>> {
    let thread = tof.into_thread(lua)?;
    let sched = lua
        .app_data_ref::<&Scheduler>()
        .expect("Lua struct is missing scheduler");
    sched.push_back(thread.clone(), args)?;
    Ok(thread)
}

// FIXME: `self` escapes outside of method because we are borrowing `tof` and
// `args` when we call `schedule_future_thread` in the lua function body below
// For now we solve this by using the 'static lifetime bound in the impl
fn task_delay<'lua>(
    lua: &'lua Lua,
    (secs, tof, args): (f64, LuaThreadOrFunction<'lua>, LuaMultiValue<'lua>),
) -> LuaResult<LuaThread<'lua>>
where
    'lua: 'static,
{
    let thread = tof.into_thread(lua)?;
    let sched = lua
        .app_data_ref::<&Scheduler>()
        .expect("Lua struct is missing scheduler");

    let thread2 = thread.clone();
    sched.spawn_thread(thread.clone(), async move {
        let duration = Duration::from_secs_f64(secs);
        time::sleep(duration).await;
        sched.push_back(thread2, args)?;
        Ok(())
    })?;

    Ok(thread)
}

async fn task_wait(_: &Lua, secs: Option<f64>) -> LuaResult<f64> {
    let duration = Duration::from_secs_f64(secs.unwrap_or_default());

    let before = Instant::now();
    time::sleep(duration).await;
    let after = Instant::now();

    Ok((after - before).as_secs_f64())
}
