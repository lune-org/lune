#![allow(clippy::too_many_lines)]

use mlua::prelude::*;

use crate::{
    error_callback::ThreadErrorCallback,
    queue::{DeferredThreadQueue, SpawnedThreadQueue},
    threads::{ThreadId, ThreadMap},
    traits::LuaSchedulerExt,
    util::{is_poll_pending, LuaThreadOrFunction},
};

const ERR_METADATA_NOT_ATTACHED: &str = "\
Lua state does not have scheduler metadata attached!\
\nThis is most likely caused by creating functions outside of a scheduler.\
\nScheduler functions must always be created from within an active scheduler.\
";

const EXIT_IMPL_LUA: &str = r"
exit(...)
yield()
";

const WRAP_IMPL_LUA: &str = r"
local t = create(...)
return function(...)
    local r = { resume(t, ...) }
    if r[1] then
        return select(2, unpack(r))
    else
        error(r[2], 2)
    end
end
";

/**
    A collection of lua functions that may be called to interact with a [`Scheduler`].

    Note that these may all be implemented using [`LuaSchedulerExt`], however, this struct
    is implemented using internal (non-public) APIs, and generally has better performance.
*/
pub struct Functions {
    /**
        Implementation of `coroutine.resume` that handles async polling properly.

        Defers onto the scheduler queue if the thread calls an async function.
    */
    pub resume: LuaFunction,
    /**
        Implementation of `coroutine.wrap` that handles async polling properly.

        Defers onto the scheduler queue if the thread calls an async function.
    */
    pub wrap: LuaFunction,
    /**
        Resumes a function / thread once instantly, and runs until first yield.

        Spawns onto the scheduler queue if not completed.
    */
    pub spawn: LuaFunction,
    /**
        Defers a function / thread onto the scheduler queue.

        Does not resume instantly, only adds to the queue.
    */
    pub defer: LuaFunction,
    /**
        Cancels a function / thread, removing it from the queue.
    */
    pub cancel: LuaFunction,
    /**
        Exits the scheduler, stopping all other threads and closing the scheduler.

        Yields the calling thread to ensure that it does not continue.
    */
    pub exit: LuaFunction,
}

impl Functions {
    /**
        Creates a new collection of Lua functions that may be called to interact with a [`Scheduler`].

        # Errors

        Errors when out of memory, or if default Lua globals are missing.

        # Panics

        Panics when the given [`Lua`] instance does not have an attached [`Scheduler`].
    */
    pub fn new(lua: Lua) -> LuaResult<Self> {
        let spawn_queue = lua
            .app_data_ref::<SpawnedThreadQueue>()
            .expect(ERR_METADATA_NOT_ATTACHED)
            .clone();
        let defer_queue = lua
            .app_data_ref::<DeferredThreadQueue>()
            .expect(ERR_METADATA_NOT_ATTACHED)
            .clone();
        let error_callback = lua
            .app_data_ref::<ThreadErrorCallback>()
            .expect(ERR_METADATA_NOT_ATTACHED)
            .clone();
        let thread_map = lua
            .app_data_ref::<ThreadMap>()
            .expect(ERR_METADATA_NOT_ATTACHED)
            .clone();

        let resume_queue = defer_queue.clone();
        let resume_map = thread_map.clone();
        let resume =
            lua.create_function(move |lua, (thread, args): (LuaThread, LuaMultiValue)| {
                let _span = tracing::trace_span!("Scheduler::fn_resume").entered();
                match thread.resume::<LuaMultiValue>(args.clone()) {
                    Ok(v) => {
                        if v.front().is_some_and(is_poll_pending) {
                            // Pending, defer to scheduler and return nil
                            resume_queue.push_item(lua, &thread, args)?;
                            (true, LuaValue::Nil).into_lua_multi(lua)
                        } else {
                            // Not pending, store the value if thread is done
                            if thread.status() != LuaThreadStatus::Resumable {
                                let id = ThreadId::from(&thread);
                                if resume_map.is_tracked(id) {
                                    resume_map.insert(id, Ok(v.clone()));
                                }
                            }
                            (true, v).into_lua_multi(lua)
                        }
                    }
                    Err(e) => {
                        // Not pending, store the error
                        let id = ThreadId::from(&thread);
                        if resume_map.is_tracked(id) {
                            resume_map.insert(id, Err(e.clone()));
                        }
                        (false, e.to_string()).into_lua_multi(lua)
                    }
                }
            })?;

        let wrap_env = lua.create_table_from(vec![
            ("resume", resume.clone()),
            ("error", lua.globals().get::<LuaFunction>("error")?),
            ("select", lua.globals().get::<LuaFunction>("select")?),
            ("unpack", lua.globals().get::<LuaFunction>("unpack")?),
            (
                "create",
                lua.globals()
                    .get::<LuaTable>("coroutine")?
                    .get::<LuaFunction>("create")?,
            ),
        ])?;
        let wrap = lua
            .load(WRAP_IMPL_LUA)
            .set_name("=__scheduler_wrap")
            .set_environment(wrap_env)
            .into_function()?;

        let spawn_map = thread_map.clone();
        let spawn = lua.create_function(
            move |lua, (tof, args): (LuaThreadOrFunction, LuaMultiValue)| {
                let _span = tracing::trace_span!("Scheduler::fn_spawn").entered();
                let thread = tof.into_thread(lua)?;
                if thread.status() == LuaThreadStatus::Resumable {
                    // NOTE: We need to resume the thread once instantly for correct behavior,
                    // and only if we get the pending value back we can spawn to async executor
                    match thread.resume::<LuaMultiValue>(args.clone()) {
                        Ok(v) => {
                            if v.front().is_some_and(is_poll_pending) {
                                spawn_queue.push_item(lua, &thread, args)?;
                            } else {
                                // Not pending, store the value if thread is done
                                if thread.status() != LuaThreadStatus::Resumable {
                                    let id = ThreadId::from(&thread);
                                    if spawn_map.is_tracked(id) {
                                        spawn_map.insert(id, Ok(v));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error_callback.call(&e);
                            // Not pending, store the error
                            let id = ThreadId::from(&thread);
                            if spawn_map.is_tracked(id) {
                                spawn_map.insert(id, Err(e));
                            }
                        }
                    }
                }
                Ok(thread)
            },
        )?;

        let defer = lua.create_function(
            move |lua, (tof, args): (LuaThreadOrFunction, LuaMultiValue)| {
                let _span = tracing::trace_span!("Scheduler::fn_defer").entered();
                let thread = tof.into_thread(lua)?;
                if thread.status() == LuaThreadStatus::Resumable {
                    defer_queue.push_item(lua, &thread, args)?;
                }
                Ok(thread)
            },
        )?;

        let close = lua
            .globals()
            .get::<LuaTable>("coroutine")?
            .get::<LuaFunction>("close")?;
        let close_key = lua.create_registry_value(close)?;
        let cancel = lua.create_function(move |lua, thread: LuaThread| {
            let _span = tracing::trace_span!("Scheduler::fn_cancel").entered();
            let close: LuaFunction = lua.registry_value(&close_key)?;
            match close.call(thread) {
                Err(LuaError::CoroutineUnresumable) | Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        })?;

        let exit_env = lua.create_table_from(vec![
            (
                "exit",
                lua.create_function(|lua, code: Option<u8>| {
                    let _span = tracing::trace_span!("Scheduler::fn_exit").entered();
                    let code = code.unwrap_or_default();
                    lua.set_exit_code(code);
                    Ok(())
                })?,
            ),
            (
                "yield",
                lua.globals()
                    .get::<LuaTable>("coroutine")?
                    .get::<LuaFunction>("yield")?,
            ),
        ])?;
        let exit = lua
            .load(EXIT_IMPL_LUA)
            .set_name("=__scheduler_exit")
            .set_environment(exit_env)
            .into_function()?;

        Ok(Self {
            resume,
            wrap,
            spawn,
            defer,
            cancel,
            exit,
        })
    }
}

impl Functions {
    /**
        Injects [`Scheduler`]-compatible functions into the given [`Lua`] instance.

        This will overwrite the following functions:

        - `coroutine.resume`
        - `coroutine.wrap`

        # Errors

        Errors when out of memory, or if default Lua globals are missing.
    */
    pub fn inject_compat(&self, lua: &Lua) -> LuaResult<()> {
        let co: LuaTable = lua.globals().get("coroutine")?;
        co.set("resume", self.resume.clone())?;
        co.set("wrap", self.wrap.clone())?;
        Ok(())
    }
}
