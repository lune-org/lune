#![allow(clippy::missing_errors_doc)]

use std::future::Future;

use mlua::prelude::*;

/**
    Utility struct for building Lua tables.
*/
pub struct TableBuilder {
    lua: Lua,
    tab: LuaTable,
}

impl TableBuilder {
    /**
        Creates a new table builder.
    */
    pub fn new(lua: Lua) -> LuaResult<Self> {
        let tab = lua.create_table()?;
        Ok(Self { lua, tab })
    }

    /**
        Adds a new key-value pair to the table.

        This will overwrite any value that already exists.
    */
    pub fn with_value<K, V>(self, key: K, value: V) -> LuaResult<Self>
    where
        K: IntoLua,
        V: IntoLua,
    {
        self.tab.raw_set(key, value)?;
        Ok(self)
    }

    /**
        Adds multiple key-value pairs to the table.

        This will overwrite any values that already exist.
    */
    pub fn with_values<K, V>(self, values: Vec<(K, V)>) -> LuaResult<Self>
    where
        K: IntoLua,
        V: IntoLua,
    {
        for (key, value) in values {
            self.tab.raw_set(key, value)?;
        }
        Ok(self)
    }

    /**
        Adds a new key-value pair to the sequential (array) section of the table.

        This will not overwrite any value that already exists,
        instead adding the value to the end of the array.
    */
    pub fn with_sequential_value<V>(self, value: V) -> LuaResult<Self>
    where
        V: IntoLua,
    {
        self.tab.raw_push(value)?;
        Ok(self)
    }

    /**
        Adds multiple values to the sequential (array) section of the table.

        This will not overwrite any values that already exist,
        instead adding the values to the end of the array.
    */
    pub fn with_sequential_values<V>(self, values: Vec<V>) -> LuaResult<Self>
    where
        V: IntoLua,
    {
        for value in values {
            self.tab.raw_push(value)?;
        }
        Ok(self)
    }

    /**
        Adds a new key-value pair to the table, with a function value.

        This will overwrite any value that already exists.
    */
    pub fn with_function<K, A, R, F>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua,
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(&Lua, A) -> LuaResult<R> + 'static,
    {
        let f = self.lua.create_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    /**
        Adds a new key-value pair to the table, with an async function value.

        This will overwrite any value that already exists.
    */
    pub fn with_async_function<K, A, R, F, FR>(self, key: K, func: F) -> LuaResult<Self>
    where
        K: IntoLua,
        A: FromLuaMulti,
        R: IntoLuaMulti,
        F: Fn(Lua, A) -> FR + 'static,
        FR: Future<Output = LuaResult<R>> + 'static,
    {
        let f = self.lua.create_async_function(func)?;
        self.with_value(key, LuaValue::Function(f))
    }

    /**
        Adds a metatable to the table.

        This will overwrite any metatable that already exists.
    */
    pub fn with_metatable(self, table: LuaTable) -> LuaResult<Self> {
        self.tab.set_metatable(Some(table))?;
        Ok(self)
    }

    /**
        Builds the table as a read-only table.

        This will prevent any *direct* modifications to the table.
    */
    pub fn build_readonly(self) -> LuaResult<LuaTable> {
        self.tab.set_readonly(true);
        Ok(self.tab)
    }

    /**
        Builds the table.
    */
    pub fn build(self) -> LuaResult<LuaTable> {
        Ok(self.tab)
    }
}
