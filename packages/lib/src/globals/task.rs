use mlua::prelude::*;

use crate::{
    lua::task::{TaskReference, TaskScheduler},
    utils::table::TableBuilder,
};

const TASK_WAIT_IMPL_LUA: &str = r#"
resume_after(thread(), ...)
return yield()
"#;

pub fn create(
    lua: &'static Lua,
    scheduler: &'static TaskScheduler,
) -> LuaResult<LuaTable<'static>> {
    lua.set_app_data(scheduler);
    // Create task spawning functions that add tasks to the scheduler
    let task_spawn = lua.create_function(|lua, (tof, args): (LuaValue, LuaMultiValue)| {
        let sched = &mut lua.app_data_mut::<&TaskScheduler>().unwrap();
        sched.schedule_instant(tof, args)
    })?;
    let task_defer = lua.create_function(|lua, (tof, args): (LuaValue, LuaMultiValue)| {
        let sched = &mut lua.app_data_mut::<&TaskScheduler>().unwrap();
        sched.schedule_deferred(tof, args)
    })?;
    let task_delay =
        lua.create_function(|lua, (secs, tof, args): (f64, LuaValue, LuaMultiValue)| {
            let sched = &mut lua.app_data_mut::<&TaskScheduler>().unwrap();
            sched.schedule_delayed(secs, tof, args)
        })?;
    // Create our task wait function, this is a bit different since
    // we have no way to yield from c / rust, we need to load a
    // lua chunk that schedules and yields for us instead
    let task_wait_env_thread: LuaFunction = lua.named_registry_value("co.thread")?;
    let task_wait_env_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let task_wait = lua
        .load(TASK_WAIT_IMPL_LUA)
        .set_environment(
            TableBuilder::new(lua)?
                .with_value("thread", task_wait_env_thread)?
                .with_value("yield", task_wait_env_yield)?
                .with_function(
                    "resume_after",
                    |lua, (thread, secs): (LuaThread, Option<f64>)| {
                        let sched = &mut lua.app_data_mut::<&TaskScheduler>().unwrap();
                        sched.resume_after(secs.unwrap_or(0f64), thread)
                    },
                )?
                .build_readonly()?,
        )?
        .into_function()?;
    // We want the task scheduler to be transparent,
    // but it does not return real lua threads, so
    // we need to override some globals to fake it
    let globals = lua.globals();
    let type_original: LuaFunction = globals.get("type")?;
    let type_proxy = lua.create_function(move |_, value: LuaValue| {
        if let LuaValue::UserData(u) = &value {
            if u.is::<TaskReference>() {
                return Ok(LuaValue::String(lua.create_string("thread")?));
            }
        }
        type_original.call(value)
    })?;
    let typeof_original: LuaFunction = globals.get("typeof")?;
    let typeof_proxy = lua.create_function(move |_, value: LuaValue| {
        if let LuaValue::UserData(u) = &value {
            if u.is::<TaskReference>() {
                return Ok(LuaValue::String(lua.create_string("thread")?));
            }
        }
        typeof_original.call(value)
    })?;
    globals.set("type", type_proxy)?;
    globals.set("typeof", typeof_proxy)?;
    // All good, return the task scheduler lib
    TableBuilder::new(lua)?
        .with_value("spawn", task_spawn)?
        .with_value("defer", task_defer)?
        .with_value("delay", task_delay)?
        .with_value("wait", task_wait)?
        .build_readonly()
}
