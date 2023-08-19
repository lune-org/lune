use std::time::Duration;

use mlua::prelude::*;

use tokio::time::{self, Instant};

use crate::lune::{scheduler::Scheduler, util::TableBuilder};

mod tof;
use tof::LuaThreadOrFunction;

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLuaMulti<'_>> {
    TableBuilder::new(lua)?
        .with_function("cancel", task_cancel)?
        .with_function("defer", task_defer)?
        .with_function("delay", task_delay)?
        .with_function("spawn", task_spawn)?
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
    sched.schedule_future_thread(thread.clone(), async move {
        let duration = Duration::from_secs_f64(secs);
        time::sleep(duration).await;
        sched.push_back(thread2, args)?;
        Ok(())
    })?;

    Ok(thread)
}

fn task_spawn<'lua>(
    lua: &'lua Lua,
    (tof, args): (LuaThreadOrFunction<'lua>, LuaMultiValue<'_>),
) -> LuaResult<LuaThread<'lua>> {
    let thread = tof.into_thread(lua)?;
    let resume = lua
        .globals()
        .get::<_, LuaTable>("coroutine")?
        .get::<_, LuaFunction>("resume")?;
    resume.call((thread.clone(), args))?;
    Ok(thread)
}

async fn task_wait(_: &Lua, secs: Option<f64>) -> LuaResult<f64> {
    let duration = Duration::from_secs_f64(secs.unwrap_or_default());

    let before = Instant::now();
    time::sleep(duration).await;
    let after = Instant::now();

    Ok((after - before).as_secs_f64())
}
