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
        Get a strong reference to the scheduler for the [`Lua`] struct.

        Note that if this reference is not dropped, `Lua` can
        not be dropped either because of the strong reference.
    */
    fn scheduler(&self) -> Scheduler;

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
    fn scheduler(&self) -> Scheduler {
        self.app_data_ref::<Scheduler>()
            .expect("Lua struct is missing scheduler")
            .clone()
    }

    fn create_async_function<'lua, A, R, F, FR>(&'lua self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        let async_yield = self
            .globals()
            .get::<_, LuaTable>("coroutine")?
            .get::<_, LuaFunction>("yield")?;
        let async_schedule = self.create_function(move |lua: &Lua, args: A| {
            let thread = lua.current_thread();
            let future = func(lua, args);
            // TODO: Add to scheduler
            Ok(())
        })?;

        let async_func = self
            .load(chunk!({
                $async_schedule(...)
                return $async_yield()
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
pub trait IntoLuaThread<'lua> {
    /**
        Converts the value into a lua thread.
    */
    fn into_lua_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>>;
}

impl<'lua> IntoLuaThread<'lua> for LuaThread<'lua> {
    fn into_lua_thread(self, _: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        Ok(self)
    }
}

impl<'lua> IntoLuaThread<'lua> for LuaFunction<'lua> {
    fn into_lua_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        lua.create_thread(self)
    }
}

impl<'lua, 'a> IntoLuaThread<'lua> for LuaChunk<'lua, 'a> {
    fn into_lua_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        lua.create_thread(self.into_function()?)
    }
}
