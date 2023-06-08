use async_trait::async_trait;
use futures_util::Future;
use mlua::prelude::*;

use crate::{lua::table::TableBuilder, lua::task::TaskScheduler};

use super::task::TaskSchedulerAsyncExt;

const ASYNC_IMPL_LUA: &str = r#"
resumeAsync(...)
return yield()
"#;

const WAIT_IMPL_LUA: &str = r#"
resumeAfter(...)
return yield()
"#;

#[async_trait(?Send)]
pub trait LuaAsyncExt {
    fn create_async_function<'lua, A, R, F, FR>(self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'static>,
        R: IntoLuaMulti<'static>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
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
        R: IntoLuaMulti<'static>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        let async_env_yield: LuaFunction = self.named_registry_value("co.yield")?;
        let async_env = TableBuilder::new(self)?
            .with_value("yield", async_env_yield)?
            .with_function("resumeAsync", move |lua: &Lua, args: A| {
                let thread = lua.current_thread();
                let fut = func(lua, args);
                let sched = lua
                    .app_data_ref::<&TaskScheduler>()
                    .expect("Missing task scheduler as a lua app data");
                sched.queue_async_task(thread, None, async {
                    let rets = fut.await?;
                    let mult = rets.into_lua_multi(lua)?;
                    Ok(Some(mult))
                })
            })?
            .build_readonly()?;
        let async_func = self
            .load(ASYNC_IMPL_LUA)
            .set_name("async")
            .set_environment(async_env)
            .into_function()?;
        Ok(async_func)
    }

    /**
        Creates a special async function that waits the
        desired amount of time, inheriting the guid of the
        current thread / task for proper cancellation.

        This will yield the lua thread calling the function until the
        desired time has passed and the scheduler resumes the thread.
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
            .load(WAIT_IMPL_LUA)
            .set_name("wait")
            .set_environment(async_env)
            .into_function()?;
        Ok(async_func)
    }
}
