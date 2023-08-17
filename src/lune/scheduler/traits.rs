use mlua::prelude::*;

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
}

impl LuaSchedulerExt for Lua {
    fn scheduler(&self) -> Scheduler {
        self.app_data_ref::<Scheduler>()
            .expect("Lua struct is missing scheduler")
            .clone()
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
