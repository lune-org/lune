use mlua::prelude::*;

use crate::lune::lua::{
    async_ext::LuaAsyncExt,
    table::TableBuilder,
    task::{
        LuaThreadOrFunction, LuaThreadOrTaskReference, TaskKind, TaskReference, TaskScheduler,
        TaskSchedulerScheduleExt,
    },
};

const SPAWN_IMPL_LUA: &str = r#"
scheduleNext(thread())
local task = scheduleNext(...)
yield()
return task
"#;

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
    let task_spawn_env_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let task_spawn = lua
        .load(SPAWN_IMPL_LUA)
        .set_name("task.spawn")
        .set_environment(
            TableBuilder::new(lua)?
                .with_function("thread", |lua, _: ()| Ok(lua.current_thread()))?
                .with_value("yield", task_spawn_env_yield)?
                .with_function(
                    "scheduleNext",
                    |lua, (tof, args): (LuaThreadOrFunction, LuaMultiValue)| {
                        let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
                        sched.schedule_blocking(tof.into_thread(lua)?, args)
                    },
                )?
                .build_readonly()?,
        )
        .into_function()?;
    // Functions in the built-in coroutine library also need to be
    // replaced, these are a bit different than the ones above because
    // calling resume or the function that wrap returns must return
    // whatever lua value(s) that the thread or task yielded back
    let globals = lua.globals();
    let coroutine = globals.get::<_, LuaTable>("coroutine")?;
    coroutine.set("status", lua.create_function(coroutine_status)?)?;
    coroutine.set("resume", lua.create_function(coroutine_resume)?)?;
    coroutine.set("wrap", lua.create_function(coroutine_wrap)?)?;
    // All good, return the task scheduler lib
    TableBuilder::new(lua)?
        .with_value("wait", lua.create_waiter_function()?)?
        .with_value("spawn", task_spawn)?
        .with_function("cancel", task_cancel)?
        .with_function("defer", task_defer)?
        .with_function("delay", task_delay)?
        .build_readonly()
}

/*
    Basic task functions
*/

fn task_cancel(lua: &Lua, task: LuaUserDataRef<TaskReference>) -> LuaResult<()> {
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    sched.remove_task(*task)?;
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

/*
    Coroutine library overrides for compat with task scheduler
*/

fn coroutine_status<'a>(
    lua: &'a Lua,
    value: LuaThreadOrTaskReference<'a>,
) -> LuaResult<LuaString<'a>> {
    Ok(match value {
        LuaThreadOrTaskReference::Thread(thread) => {
            let get_status: LuaFunction = lua.named_registry_value("co.status")?;
            get_status.call(thread)?
        }
        LuaThreadOrTaskReference::TaskReference(task) => {
            let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
            sched
                .get_task_status(task)
                .unwrap_or_else(|| lua.create_string("dead").unwrap())
        }
    })
}

fn coroutine_resume<'lua>(
    lua: &'lua Lua,
    (value, arguments): (LuaThreadOrTaskReference, LuaMultiValue<'lua>),
) -> LuaResult<(bool, LuaMultiValue<'lua>)> {
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    if sched.current_task().is_none() {
        return Err(LuaError::RuntimeError(
            "No current task to inherit".to_string(),
        ));
    }
    let current = sched.current_task().unwrap();
    let result = match value {
        LuaThreadOrTaskReference::Thread(t) => {
            let task = sched.create_task(TaskKind::Instant, t, Some(arguments), true)?;
            sched.resume_task(task, None)
        }
        LuaThreadOrTaskReference::TaskReference(t) => sched.resume_task(t, Some(Ok(arguments))),
    };
    sched.force_set_current_task(Some(current));
    match result {
        Ok(rets) => Ok((true, rets.1)),
        Err(e) => Ok((false, e.into_lua_multi(lua)?)),
    }
}

fn coroutine_wrap<'lua>(lua: &'lua Lua, func: LuaFunction) -> LuaResult<LuaFunction<'lua>> {
    let task = lua.app_data_ref::<&TaskScheduler>().unwrap().create_task(
        TaskKind::Instant,
        lua.create_thread(func)?,
        None,
        false,
    )?;
    lua.create_function(move |lua, args: LuaMultiValue| {
        let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
        if sched.current_task().is_none() {
            return Err(LuaError::RuntimeError(
                "No current task to inherit".to_string(),
            ));
        }
        let current = sched.current_task().unwrap();
        let result = lua
            .app_data_ref::<&TaskScheduler>()
            .unwrap()
            .resume_task(task, Some(Ok(args)));
        sched.force_set_current_task(Some(current));
        match result {
            Ok(rets) => Ok(rets.1),
            Err(e) => Err(e),
        }
    })
}
