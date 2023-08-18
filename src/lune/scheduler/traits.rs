use futures_util::Future;
use mlua::{chunk, prelude::*};

use super::Scheduler;

/**
    Trait for extensions to the [`Lua`] struct, allowing
    for access to the scheduler without having to import
    it or handle registry / app data references manually.
*/
pub trait LuaSchedulerExt {
    /**
        Get a reference to the scheduler for the [`Lua`] struct.
    */
    fn scheduler(&self) -> &Scheduler;

    /**
        Creates a function callable from Lua that runs an async
        closure and returns the results of it to the call site.
    */
    fn create_async_function<'lua, A, R, F, FR>(
        &'lua self,
        func: F,
    ) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>;
}

impl LuaSchedulerExt for Lua {
    fn scheduler(&self) -> &Scheduler {
        *self
            .app_data_ref::<&Scheduler>()
            .expect("Lua struct is missing scheduler")
    }

    fn create_async_function<'lua, A, R, F, FR>(&'lua self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        let coroutine_yield = self
            .globals()
            .get::<_, LuaTable>("coroutine")?
            .get::<_, LuaFunction>("yield")?;
        let schedule = LuaFunction::wrap(move |lua: &Lua, args: A| {
            let thread = lua.current_thread().into_owned();
            let future = func(lua, args);
            lua.scheduler().schedule_future_thread(thread, future);
            Ok(())
        });

        let async_func = self
            .load(chunk!({
                $schedule(...)
                return $coroutine_yield()
            }))
            .set_name("async")
            .into_function()?;
        Ok(async_func)
    }
}

/**
    Trait for any struct that can be turned into an [`LuaThread`]
    and given to the scheduler, implemented for the following types:

    - Lua threads ([`LuaThread`])
    - Lua functions ([`LuaFunction`])
    - Lua chunks ([`LuaChunk`])
*/
pub trait IntoLuaOwnedThread {
    /**
        Converts the value into a lua thread.
    */
    fn into_owned_lua_thread(self, lua: &Lua) -> LuaResult<LuaOwnedThread>;
}

impl IntoLuaOwnedThread for LuaOwnedThread {
    fn into_owned_lua_thread(self, _lua: &Lua) -> LuaResult<LuaOwnedThread> {
        Ok(self)
    }
}

impl<'lua> IntoLuaOwnedThread for LuaThread<'lua> {
    fn into_owned_lua_thread(self, _lua: &Lua) -> LuaResult<LuaOwnedThread> {
        Ok(self.into_owned())
    }
}

impl<'lua> IntoLuaOwnedThread for LuaFunction<'lua> {
    fn into_owned_lua_thread(self, lua: &Lua) -> LuaResult<LuaOwnedThread> {
        Ok(lua.create_thread(self)?.into_owned())
    }
}

impl<'lua, 'a> IntoLuaOwnedThread for LuaChunk<'lua, 'a> {
    fn into_owned_lua_thread(self, lua: &Lua) -> LuaResult<LuaOwnedThread> {
        Ok(lua.create_thread(self.into_function()?)?.into_owned())
    }
}
