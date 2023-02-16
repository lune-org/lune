use async_trait::async_trait;
use futures_util::Future;
use mlua::prelude::*;

use crate::{lua::task::TaskScheduler, utils::table::TableBuilder};

#[async_trait(?Send)]
pub trait LuaAsyncExt {
    fn create_async_function<'lua, A, R, F, FR>(self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'static>,
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>;
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
        let async_env_thread: LuaFunction = self.named_registry_value("co.thread")?;
        let async_env_yield: LuaFunction = self.named_registry_value("co.yield")?;
        let async_env = TableBuilder::new(self)?
            .with_value("thread", async_env_thread)?
            .with_value("yield", async_env_yield)?
            .with_function(
                "resumeAsync",
                move |lua: &Lua, (thread, args): (LuaThread, A)| {
                    let fut = func(lua, args);
                    let sched = lua
                        .app_data_ref::<&TaskScheduler>()
                        .expect("Missing task scheduler as a lua app data");
                    sched.queue_async_task(thread, None, None, async {
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
                return yield()
                ",
            )
            .set_name("asyncWrapper")?
            .set_environment(async_env)?
            .into_function()?;
        Ok(async_func)
    }
}
