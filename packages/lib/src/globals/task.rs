use mlua::prelude::*;

use crate::{
    lua::task::{TaskKind, TaskReference, TaskScheduler, TaskSchedulerScheduleExt},
    utils::table::TableBuilder,
};

const ERR_MISSING_SCHEDULER: &str = "Missing task scheduler - make sure it is added as a lua app data before the first scheduler resumption";

const TASK_WAIT_IMPL_LUA: &str = r#"
local seconds = ...
local current = thread()
resumeAfter(seconds, current)
return yield()
"#;

const TASK_SPAWN_IMPL_LUA: &str = r#"
-- Schedule the current thread at the front
scheduleNext(thread())
-- Schedule the wanted task arg at the front,
-- the previous schedule now comes right after
local task = scheduleNext(...)
-- Give control over to the scheduler, which will
-- resume the above tasks in order when its ready
yield()
return task
"#;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable<'static>> {
    // Create a user-accessible function that cancels a task
    let task_cancel = lua.create_function(|lua, task: TaskReference| {
        let sched = lua
            .app_data_ref::<&TaskScheduler>()
            .expect(ERR_MISSING_SCHEDULER);
        sched.remove_task(task)?;
        Ok(())
    })?;
    // Create functions that manipulate non-blocking tasks in the scheduler
    let task_defer = lua.create_function(|lua, (tof, args): (LuaValue, LuaMultiValue)| {
        let sched = lua
            .app_data_ref::<&TaskScheduler>()
            .expect(ERR_MISSING_SCHEDULER);
        sched.schedule_blocking_deferred(tof, args)
    })?;
    let task_delay =
        lua.create_function(|lua, (secs, tof, args): (f64, LuaValue, LuaMultiValue)| {
            let sched = lua
                .app_data_ref::<&TaskScheduler>()
                .expect(ERR_MISSING_SCHEDULER);
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
                    "resumeAfter",
                    |lua, (secs, thread): (Option<f64>, LuaThread)| {
                        let sched = lua
                            .app_data_ref::<&TaskScheduler>()
                            .expect(ERR_MISSING_SCHEDULER);
                        sched.schedule_wait(secs.unwrap_or_default(), LuaValue::Thread(thread))
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
                    "scheduleNext",
                    |lua, (tof, args): (LuaValue, LuaMultiValue)| {
                        let sched = lua
                            .app_data_ref::<&TaskScheduler>()
                            .expect(ERR_MISSING_SCHEDULER);
                        sched.schedule_blocking(tof, args)
                    },
                )?
                .build_readonly()?,
        )?
        .into_function()?;
    // We want the task scheduler to be transparent,
    // but it does not return real lua threads, so
    // we need to override some globals to fake it
    let type_original: LuaFunction = lua.named_registry_value("type")?;
    let type_proxy = lua.create_function(move |_, value: LuaValue| {
        if let LuaValue::UserData(u) = &value {
            if u.is::<TaskReference>() {
                return Ok(LuaValue::String(lua.create_string("thread")?));
            }
        }
        type_original.call(value)
    })?;
    let typeof_original: LuaFunction = lua.named_registry_value("typeof")?;
    let typeof_proxy = lua.create_function(move |_, value: LuaValue| {
        if let LuaValue::UserData(u) = &value {
            if u.is::<TaskReference>() {
                return Ok(LuaValue::String(lua.create_string("thread")?));
            }
        }
        typeof_original.call(value)
    })?;
    let globals = lua.globals();
    globals.set("type", type_proxy)?;
    globals.set("typeof", typeof_proxy)?;
    // Functions in the built-in coroutine library also need to be
    // replaced, these are a bit different than the ones above because
    // calling resume or the function that wrap returns must return
    // whatever lua value(s) that the thread or task yielded back
    let coroutine = globals.get::<_, LuaTable>("coroutine")?;
    coroutine.set(
        "resume",
        lua.create_function(|lua, value: LuaValue| {
            let tname = value.type_name();
            if let LuaValue::Thread(thread) = value {
                let sched = lua
                    .app_data_ref::<&TaskScheduler>()
                    .expect(ERR_MISSING_SCHEDULER);
                let task =
                    sched.create_task(TaskKind::Instant, LuaValue::Thread(thread), None, None)?;
                sched.resume_task(task, None)
            } else if let Ok(task) = TaskReference::from_lua(value, lua) {
                lua.app_data_ref::<&TaskScheduler>()
                    .expect(ERR_MISSING_SCHEDULER)
                    .resume_task(task, None)
            } else {
                Err(LuaError::RuntimeError(format!(
                    "Argument #1 must be a thread, got {tname}",
                )))
            }
        })?,
    )?;
    coroutine.set(
        "wrap",
        lua.create_function(|lua, func: LuaFunction| {
            let sched = lua
                .app_data_ref::<&TaskScheduler>()
                .expect(ERR_MISSING_SCHEDULER);
            let task =
                sched.create_task(TaskKind::Instant, LuaValue::Function(func), None, None)?;
            lua.create_function(move |lua, args: LuaMultiValue| {
                let sched = lua
                    .app_data_ref::<&TaskScheduler>()
                    .expect(ERR_MISSING_SCHEDULER);
                sched.resume_task(task, Some(Ok(args)))
            })
        })?,
    )?;
    // All good, return the task scheduler lib
    TableBuilder::new(lua)?
        .with_value("cancel", task_cancel)?
        .with_value("spawn", task_spawn)?
        .with_value("defer", task_defer)?
        .with_value("delay", task_delay)?
        .with_value("wait", task_wait)?
        .build_readonly()
}
