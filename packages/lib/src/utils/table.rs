use std::future::Future;

use mlua::prelude::*;

pub struct TableBuilder<'lua> {
    lua: &'lua Lua,
    tab: LuaTable<'lua>,
}

#[allow(dead_code)]
impl<'lua> TableBuilder<'lua> {
    pub fn new(lua: &'lua Lua) -> LuaResult<Self> {
        let tab = lua.create_table()?;
        Ok(Self { lua, tab })
    }

    pub fn with_value<K, V>(self, key: K, value: V) -> LuaResult<Self>
    where
        K: ToLua<'lua>,
        V: ToLua<'lua>,
    {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    pub fn with_values<K, V>(self, values: Vec<(K, V)>) -> LuaResult<Self>
    where
        K: ToLua<'lua>,
        V: ToLua<'lua>,
    {
        for (key, value) in values {
            self.tab.raw_set(key, value)?;
        }
        Ok(self)
    }

    pub fn with_sequential_value<V>(self, value: V) -> LuaResult<Self>
    where
        V: ToLua<'lua>,
    {
        self.tab.raw_push(value)?;
        Ok(self)
    }

    pub fn with_sequential_values<V>(self, values: Vec<V>) -> LuaResult<Self>
    where
        V: ToLua<'lua>,
    {
        for value in values {
            self.tab.raw_push(value)?;
        }
        Ok(self)
    }

    pub fn with_metatable(self, table: LuaTable) -> LuaResult<Self> {
        self.tab.set_metatable(Some(table));
        Ok(self)
    }

    pub fn with_function<K, A, R, F>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: ToLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> LuaResult<R>,
    {
        let f = self.lua.create_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    pub fn with_async_function<K, A, R, F, FR>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: ToLua<'lua>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + Future<Output = LuaResult<R>>,
    {
        let f = self.lua.create_async_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    pub fn build_readonly(self) -> LuaResult<LuaTable<'lua>> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }

    pub fn build(self) -> LuaResult<LuaTable<'lua>> {
        Ok(self.tab)
    }
}
