use std::future::Future;

use mlua::{FromLuaMulti, Lua, Result, Table, ToLuaMulti, Value};

pub struct ReadonlyTableBuilder<'lua> {
    lua: &'lua Lua,
    tab: Table<'lua>,
}

impl<'lua> ReadonlyTableBuilder<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self> {
        let tab = lua.create_table()?;
        Ok(Self { lua, tab })
    }

    pub fn with_value(self, key: &'static str, value: Value) -> Result<Self> {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_table(self, key: &'static str, value: Table) -> Result<Self> {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_function<A, R, F>(self, key: &'static str, func: F) -> Result<Self>
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> Result<R>,
    {
        let value = self.lua.create_function(func)?;
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_async_function<A, R, F, FR>(self, key: &'static str, func: F) -> Result<Self>
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + Future<Output = Result<R>>,
    {
        let value = self.lua.create_async_function(func)?;
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn build(self) -> Result<Table<'lua>> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }
}
