use mlua::prelude::*;

use crate::{
    lua::task::{TaskReference, TaskScheduler},
    utils::table::TableBuilder,
};

const TASK_WAIT_IMPL_LUA: &str = r#"
resume_after(thread(), ...)
return yield()
"#;

const TASK_SPAWN_IMPL_LUA: &str = r#"
local task = resume_first(...)
resume_second(thread())
yield()
return task
"#;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable<'static>> {
    // Create task spawning functions that add tasks to the scheduler
    let task_cancel = lua.create_function(|lua, task: TaskReference| {
        let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
        sched.cancel_task(task)?;
        Ok(())
    })?;
    let task_defer = lua.create_function(|lua, (tof, args): (LuaValue, LuaMultiValue)| {
        let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
        sched.schedule_deferred(tof, args)
    })?;
    let task_delay =
        lua.create_function(|lua, (secs, tof, args): (f64, LuaValue, LuaMultiValue)| {
            let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
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
                        let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
                        sched.schedule_wait(secs.unwrap_or(0f64), LuaValue::Thread(thread))
                    },
                )?
                .build_readonly()?,
        )?
        .into_function()?;
    // The spawn function also needs special treatment,
    // we need to yield right away to allow the
    // spawned task to run until first yield
    let task_spawn_env_thread: LuaFunction = lua.named_registry_value("co.thread")?;
    let task_spawn_env_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let task_spawn = lua
        .load(TASK_SPAWN_IMPL_LUA)
        .set_environment(
            TableBuilder::new(lua)?
                .with_value("thread", task_spawn_env_thread)?
                .with_value("yield", task_spawn_env_yield)?
                .with_function(
                    "resume_first",
                    |lua, (tof, args): (LuaValue, LuaMultiValue)| {
                        let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
                        sched.schedule_current_resume(tof, args)
                    },
                )?
                .with_function("resume_second", |lua, thread: LuaThread| {
                    let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
                    sched.schedule_after_current_resume(
                        LuaValue::Thread(thread),
                        LuaMultiValue::new(),
                    )
                })?
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
        .with_value("cancel", task_cancel)?
        .with_value("spawn", task_spawn)?
        .with_value("defer", task_defer)?
        .with_value("delay", task_delay)?
        .with_value("wait", task_wait)?
        .build_readonly()
}
