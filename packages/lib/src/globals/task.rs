use std::time::Duration;

use mlua::prelude::*;
use tokio::time::{sleep, Instant};

use crate::{
    lua::task::{TaskKind, TaskReference, TaskScheduler, TaskSchedulerScheduleExt},
    utils::table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable<'static>> {
    lua.app_data_ref::<&TaskScheduler>()
        .expect("Missing task scheduler in app data");
    /*
        1. Schedule the current thread at the front
        2. Schedule the wanted task arg at the front,
           the previous schedule now comes right after
        3. Give control over to the scheduler, which will
           resume the above tasks in order when its ready

        The spawn function needs special treatment,
        we need to yield right away to allow the
        spawned task to run until first yield
    */
    let task_spawn_env_thread: LuaFunction = lua.named_registry_value("co.thread")?;
    let task_spawn_env_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let task_spawn = lua
        .load(
            "
            scheduleNext(thread())
            local task = scheduleNext(...)
            yield()
            return task
            ",
        )
        .set_name("=task.spawn")?
        .set_environment(
            TableBuilder::new(lua)?
                .with_value("thread", task_spawn_env_thread)?
                .with_value("yield", task_spawn_env_yield)?
                .with_function(
                    "scheduleNext",
                    |lua, (tof, args): (LuaThreadOrFunction, LuaMultiValue)| {
                        let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
                        sched.schedule_blocking(tof.into_thread(lua)?, args)
                    },
                )?
                .build_readonly()?,
        )?
        .into_function()?;
    // We want the task scheduler to be transparent,
    // but it does not return real lua threads, so
    // we need to override some globals to fake it
    let globals = lua.globals();
    globals.set("type", lua.create_function(proxy_type)?)?;
    globals.set("typeof", lua.create_function(proxy_typeof)?)?;
    // Functions in the built-in coroutine library also need to be
    // replaced, these are a bit different than the ones above because
    // calling resume or the function that wrap returns must return
    // whatever lua value(s) that the thread or task yielded back
    let coroutine = globals.get::<_, LuaTable>("coroutine")?;
    coroutine.set("resume", lua.create_function(coroutine_resume)?)?;
    coroutine.set("wrap", lua.create_function(coroutine_wrap)?)?;
    // All good, return the task scheduler lib
    TableBuilder::new(lua)?
        .with_value("spawn", task_spawn)?
        .with_function("cancel", task_cancel)?
        .with_function("defer", task_defer)?
        .with_function("delay", task_delay)?
        .with_async_function("wait", task_wait)?
        .build_readonly()
}

/*
    Proxy enum to deal with both threads & functions
*/

enum LuaThreadOrFunction<'lua> {
    Thread(LuaThread<'lua>),
    Function(LuaFunction<'lua>),
}

impl<'lua> LuaThreadOrFunction<'lua> {
    fn into_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        match self {
            Self::Thread(t) => Ok(t),
            Self::Function(f) => lua.create_thread(f),
        }
    }
}

impl<'lua> FromLua<'lua> for LuaThreadOrFunction<'lua> {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Thread(t) => Ok(Self::Thread(t)),
            LuaValue::Function(f) => Ok(Self::Function(f)),
            value => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaThreadOrFunction",
                message: Some(format!(
                    "Expected thread or function, got '{}'",
                    value.type_name()
                )),
            }),
        }
    }
}

/*
    Proxy enum to deal with both threads & task scheduler task references
*/

enum LuaThreadOrTaskReference<'lua> {
    Thread(LuaThread<'lua>),
    TaskReference(TaskReference),
}

impl<'lua> FromLua<'lua> for LuaThreadOrTaskReference<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let tname = value.type_name();
        match value {
            LuaValue::Thread(t) => Ok(Self::Thread(t)),
            LuaValue::UserData(u) => {
                if let Ok(task) = TaskReference::from_lua(LuaValue::UserData(u), lua) {
                    Ok(Self::TaskReference(task))
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: tname,
                        to: "thread",
                        message: Some(format!("Expected thread, got '{tname}'")),
                    })
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: tname,
                to: "thread",
                message: Some(format!("Expected thread, got '{tname}'")),
            }),
        }
    }
}

/*
    Basic task functions
*/

fn task_cancel(lua: &Lua, task: TaskReference) -> LuaResult<()> {
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    sched.remove_task(task)?;
    Ok(())
}

fn task_defer(
    lua: &Lua,
    (tof, args): (LuaThreadOrFunction, LuaMultiValue),
) -> LuaResult<TaskReference> {
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    sched.schedule_blocking_deferred(tof.into_thread(lua)?, args)
}

fn task_delay(
    lua: &Lua,
    (secs, tof, args): (f64, LuaThreadOrFunction, LuaMultiValue),
) -> LuaResult<TaskReference> {
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    sched.schedule_blocking_after_seconds(secs, tof.into_thread(lua)?, args)
}

async fn task_wait(_: &Lua, secs: Option<f64>) -> LuaResult<f64> {
    let start = Instant::now();
    sleep(Duration::from_secs_f64(secs.unwrap_or_default())).await;
    Ok(start.elapsed().as_secs_f64())
}

/*
    Type getter overrides for compat with task scheduler
*/

fn proxy_type<'lua>(lua: &'lua Lua, value: LuaValue<'lua>) -> LuaResult<LuaString<'lua>> {
    if let LuaValue::UserData(u) = &value {
        if u.is::<TaskReference>() {
            return lua.create_string("thread");
        }
    }
    lua.named_registry_value::<_, LuaFunction>("type")?
        .call(value)
}

fn proxy_typeof<'lua>(lua: &'lua Lua, value: LuaValue<'lua>) -> LuaResult<LuaString<'lua>> {
    if let LuaValue::UserData(u) = &value {
        if u.is::<TaskReference>() {
            return lua.create_string("thread");
        }
    }
    lua.named_registry_value::<_, LuaFunction>("typeof")?
        .call(value)
}

/*
    Coroutine library overrides for compat with task scheduler
*/

fn coroutine_resume<'lua>(
    lua: &'lua Lua,
    value: LuaThreadOrTaskReference,
) -> LuaResult<LuaMultiValue<'lua>> {
    match value {
        LuaThreadOrTaskReference::Thread(t) => {
            let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
            let task = sched.create_task(TaskKind::Instant, t, None, None)?;
            sched.resume_task(task, None)
        }
        LuaThreadOrTaskReference::TaskReference(t) => lua
            .app_data_ref::<&TaskScheduler>()
            .unwrap()
            .resume_task(t, None),
    }
}

fn coroutine_wrap<'lua>(lua: &'lua Lua, func: LuaFunction) -> LuaResult<LuaFunction<'lua>> {
    let task = lua.app_data_ref::<&TaskScheduler>().unwrap().create_task(
        TaskKind::Instant,
        lua.create_thread(func)?,
        None,
        None,
    )?;
    lua.create_function(move |lua, args: LuaMultiValue| {
        lua.app_data_ref::<&TaskScheduler>()
            .unwrap()
            .resume_task(task, Some(Ok(args)))
    })
}
