use async_trait::async_trait;
use futures_util::Future;
use mlua::prelude::*;

use crate::{lua::task::TaskScheduler, utils::table::TableBuilder};

use super::task::TaskSchedulerAsyncExt;

#[async_trait(?Send)]
pub trait LuaAsyncExt {
    fn create_async_function<'lua, A, R, F, FR>(self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'static>,
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>;

    fn create_waiter_function<'lua>(self) -> LuaResult<LuaFunction<'lua>>;
}

impl LuaAsyncExt for &'static Lua {
    /**
        Creates a function callable from Lua that runs an async
        closure and returns the results of it to the call site.
    */
    fn create_async_function<'lua, A, R, F, FR>(self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'static>,
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        let async_env_make_err: LuaFunction = self.named_registry_value("dbg.makeerr")?;
        let async_env_is_err: LuaFunction = self.named_registry_value("dbg.iserr")?;
        let async_env_trace: LuaFunction = self.named_registry_value("dbg.trace")?;
        let async_env_error: LuaFunction = self.named_registry_value("error")?;
        let async_env_unpack: LuaFunction = self.named_registry_value("tab.unpack")?;
        let async_env_yield: LuaFunction = self.named_registry_value("co.yield")?;
        let async_env = TableBuilder::new(self)?
            .with_value("makeError", async_env_make_err)?
            .with_value("isError", async_env_is_err)?
            .with_value("trace", async_env_trace)?
            .with_value("error", async_env_error)?
            .with_value("unpack", async_env_unpack)?
            .with_value("yield", async_env_yield)?
            .with_function("thread", |lua, _: ()| Ok(lua.current_thread()))?
            .with_function(
                "resumeAsync",
                move |lua: &Lua, (thread, args): (LuaThread, A)| {
                    let fut = func(lua, args);
                    let sched = lua
                        .app_data_ref::<&TaskScheduler>()
                        .expect("Missing task scheduler as a lua app data");
                    sched.queue_async_task(thread, None, async {
                        let rets = fut.await?;
                        let mult = rets.to_lua_multi(lua)?;
                        Ok(Some(mult))
                    })
                },
            )?
            .build_readonly()?;
        let async_func = self
            .load(
                "
                resumeAsync(thread(), ...)
                local results = { yield() }
                if isError(results[1]) then
                    error(makeError(results[1], trace()))
                else
                    return unpack(results)
                end
                ",
            )
            .set_name("async")?
            .set_environment(async_env)?
            .into_function()?;
        Ok(async_func)
    }

    /**
        Creates a special async function that waits the
        desired amount of time, inheriting the guid of the
        current thread / task for proper cancellation.
    */
    fn create_waiter_function<'lua>(self) -> LuaResult<LuaFunction<'lua>> {
        let async_env_yield: LuaFunction = self.named_registry_value("co.yield")?;
        let async_env = TableBuilder::new(self)?
            .with_value("yield", async_env_yield)?
            .with_function("resumeAfter", move |lua: &Lua, duration: Option<f64>| {
                let sched = lua
                    .app_data_ref::<&TaskScheduler>()
                    .expect("Missing task scheduler as a lua app data");
                sched.schedule_wait(lua.current_thread(), duration)
            })?
            .build_readonly()?;
        let async_func = self
            .load(
                "
                resumeAfter(...)
                return yield()
                ",
            )
            .set_name("wait")?
            .set_environment(async_env)?
            .into_function()?;
        Ok(async_func)
    }
}
