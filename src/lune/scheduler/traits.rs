use futures_util::Future;
use mlua::prelude::*;

use super::Scheduler;

const ASYNC_IMPL_LUA: &str = r#"
schedule(...)
return yield()
"#;

/**
    Trait for extensions to the [`Lua`] struct, allowing
    for access to the scheduler without having to import
    it or handle registry / app data references manually.
*/
pub trait LuaSchedulerExt<'lua, 'fut>
where
    'lua: 'fut,
{
    /**
        Creates a function callable from Lua that runs an async
        closure and returns the results of it to the call site.
    */
    fn create_async_function<A, R, F, FR>(&'lua self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'fut + Future<Output = LuaResult<R>>;
}

impl<'lua, 'fut> LuaSchedulerExt<'lua, 'fut> for Lua
where
    'lua: 'fut,
{
    fn create_async_function<A, R, F, FR>(&'lua self, func: F) -> LuaResult<LuaFunction<'lua>>
    where
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'fut + Future<Output = LuaResult<R>>,
    {
        let async_env = self.create_table_with_capacity(0, 2)?;

        async_env.set(
            "yield",
            self.globals()
                .get::<_, LuaTable>("coroutine")?
                .get::<_, LuaFunction>("yield")?,
        )?;

        async_env.set(
            "schedule",
            LuaFunction::wrap(move |lua: &Lua, args: A| {
                let _thread = lua.current_thread().into_owned();
                let _future = func(lua, args);
                let _sched = lua
                    .app_data_ref::<&Scheduler>()
                    .expect("Lua struct is missing scheduler");
                // FIXME: `self` escapes outside of method
                // sched.schedule_future_thread(thread, future)?;
                Ok(())
            }),
        )?;

        let async_func = self
            .load(ASYNC_IMPL_LUA)
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
