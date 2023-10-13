#![allow(dead_code)]

use std::future::Future;

use mlua::prelude::*;

use crate::lune::scheduler::LuaSchedulerExt;

pub struct TableBuilder<'lua> {
    lua: &'lua Lua,
    tab: LuaTable<'lua>,
}

impl<'lua> TableBuilder<'lua> {
    pub fn new(lua: &'lua Lua) -> LuaResult<Self> {
        let tab = lua.create_table()?;
        Ok(Self { lua, tab })
    }

    pub fn with_value<K, V>(self, key: K, value: V) -> LuaResult<Self>
    where
        K: IntoLua<'lua>,
        V: IntoLua<'lua>,
    {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_values<K, V>(self, values: Vec<(K, V)>) -> LuaResult<Self>
    where
        K: IntoLua<'lua>,
        V: IntoLua<'lua>,
    {
        for (key, value) in values {
            self.tab.raw_set(key, value)?;
        }
        Ok(self)
    }

    pub fn with_sequential_value<V>(self, value: V) -> LuaResult<Self>
    where
        V: IntoLua<'lua>,
    {
        self.tab.raw_push(value)?;
        Ok(self)
    }

    pub fn with_sequential_values<V>(self, values: Vec<V>) -> LuaResult<Self>
    where
        V: IntoLua<'lua>,
    {
        for value in values {
            self.tab.raw_push(value)?;
        }
        Ok(self)
    }

    pub fn with_function<K, A, R, F>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: Fn(&'lua Lua, A) -> LuaResult<R> + 'static,
    {
        let f = self.lua.create_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    pub fn with_table<K>(self, key: K, table: LuaTable<'lua>) -> LuaResult<Self>
    where
        K: IntoLua<'lua>,
    {
        self.tab.raw_set(key, table)?;

        Ok(self)
    }

    pub fn with_metatable(self, table: LuaTable) -> LuaResult<Self> {
        self.tab.set_metatable(Some(table));
        Ok(self)
    }

    pub fn build_readonly(self) -> LuaResult<LuaTable<'lua>> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }

    pub fn build(self) -> LuaResult<LuaTable<'lua>> {
        Ok(self.tab)
    }
}

// FIXME: Remove static lifetime bound here when `create_async_function`
// no longer needs it to compile, then move this into the above impl
impl<'lua> TableBuilder<'lua>
where
    'lua: 'static,
{
    pub fn with_async_function<K, A, R, F, FR>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: IntoLuaMulti<'lua>,
        F: Fn(&'lua Lua, A) -> FR + 'lua,
        FR: Future<Output = LuaResult<R>> + 'lua,
    {
        let f = self.lua.create_async_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }
}
