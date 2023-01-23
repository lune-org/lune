use std::future::Future;

use mlua::{FromLuaMulti, Lua, Result, Table, ToLua, ToLuaMulti, Value};

pub struct TableBuilder<'lua> {
    lua: &'lua Lua,
    tab: Table<'lua>,
}

impl<'lua> TableBuilder<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self> {
        let tab = lua.create_table()?;
        Ok(Self { lua, tab })
    }

    pub fn with_value<K, V>(self, key: K, value: V) -> Result<Self>
    where
        K: ToLua<'lua>,
        V: ToLua<'lua>,
    {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_values<K, V>(self, values: Vec<(K, V)>) -> Result<Self>
    where
        K: ToLua<'lua>,
        V: ToLua<'lua>,
    {
        for (key, value) in values {
            self.tab.raw_set(key, value)?;
        }
        Ok(self)
    }

    pub fn with_sequential_value<V>(self, value: V) -> Result<Self>
    where
        V: ToLua<'lua>,
    {
        self.tab.raw_push(value)?;
        Ok(self)
    }

    pub fn with_sequential_values<V>(self, values: Vec<V>) -> Result<Self>
    where
        V: ToLua<'lua>,
    {
        for value in values {
            self.tab.raw_push(value)?;
        }
        Ok(self)
    }

    pub fn with_metatable(self, table: Table) -> Result<Self> {
        self.tab.set_metatable(Some(table));
        Ok(self)
    }

    pub fn with_function<K, A, R, F>(self, key: K, func: F) -> Result<Self>
    where
        K: ToLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> Result<R>,
    {
        let f = self.lua.create_function(func)?;
        self.with_value(key, Value::Function(f))
    }

    pub fn with_async_function<K, A, R, F, FR>(self, key: K, func: F) -> Result<Self>
    where
        K: ToLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + Future<Output = Result<R>>,
    {
        let f = self.lua.create_async_function(func)?;
        self.with_value(key, Value::Function(f))
    }

    pub fn build_readonly(self) -> Result<Table<'lua>> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }

    pub fn build(self) -> Result<Table<'lua>> {
        Ok(self.tab)
    }
}
