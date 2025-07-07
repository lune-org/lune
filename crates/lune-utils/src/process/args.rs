#![allow(clippy::missing_panics_doc)]

use std::{
    env::args_os,
    ffi::OsString,
    sync::{Arc, Mutex},
};

use mlua::prelude::*;
use os_str_bytes::OsStringBytes;

// Inner (shared) struct

#[derive(Debug, Default)]
struct ProcessArgsInner {
    values: Vec<OsString>,
}

impl FromIterator<OsString> for ProcessArgsInner {
    fn from_iter<T: IntoIterator<Item = OsString>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

/**
    A struct that can be easily shared, stored in Lua app data,
    and that also guarantees the values are valid OS strings
    that can be used for process arguments.

    Usable directly from Lua, implementing both `FromLua` and `LuaUserData`.

    Also provides convenience methods for working with the arguments
    as either `OsString` or `Vec<u8>`, where using the latter implicitly
    converts to an `OsString` and fails if the conversion is not possible.
*/
#[derive(Debug, Clone)]
pub struct ProcessArgs {
    inner: Arc<Mutex<ProcessArgsInner>>,
}

impl ProcessArgs {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProcessArgsInner::default())),
        }
    }

    #[must_use]
    pub fn current() -> Self {
        Self {
            inner: Arc::new(Mutex::new(args_os().collect())),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.values.is_empty()
    }

    // OS strings

    #[must_use]
    pub fn all(&self) -> Vec<OsString> {
        let inner = self.inner.lock().unwrap();
        inner.values.clone()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<OsString> {
        let inner = self.inner.lock().unwrap();
        inner.values.get(index).cloned()
    }

    pub fn set(&self, index: usize, val: impl Into<OsString>) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(arg) = inner.values.get_mut(index) {
            *arg = val.into();
        }
    }

    pub fn push(&self, val: impl Into<OsString>) {
        let mut inner = self.inner.lock().unwrap();
        inner.values.push(val.into());
    }

    #[must_use]
    pub fn pop(&self) -> Option<OsString> {
        let mut inner = self.inner.lock().unwrap();
        inner.values.pop()
    }

    pub fn insert(&self, index: usize, val: impl Into<OsString>) {
        let mut inner = self.inner.lock().unwrap();
        if index <= inner.values.len() {
            inner.values.insert(index, val.into());
        }
    }

    #[must_use]
    pub fn remove(&self, index: usize) -> Option<OsString> {
        let mut inner = self.inner.lock().unwrap();
        if index < inner.values.len() {
            Some(inner.values.remove(index))
        } else {
            None
        }
    }

    // Bytes wrappers

    #[must_use]
    pub fn all_bytes(&self) -> Vec<Vec<u8>> {
        self.all()
            .into_iter()
            .filter_map(OsString::into_io_vec)
            .collect()
    }

    #[must_use]
    pub fn get_bytes(&self, index: usize) -> Option<Vec<u8>> {
        let val = self.get(index)?;
        val.into_io_vec()
    }

    pub fn set_bytes(&self, index: usize, val: impl Into<Vec<u8>>) {
        if let Some(val_os) = OsString::from_io_vec(val.into()) {
            self.set(index, val_os);
        }
    }

    pub fn push_bytes(&self, val: impl Into<Vec<u8>>) {
        if let Some(val_os) = OsString::from_io_vec(val.into()) {
            self.push(val_os);
        }
    }

    #[must_use]
    pub fn pop_bytes(&self) -> Option<Vec<u8>> {
        self.pop().and_then(OsString::into_io_vec)
    }

    pub fn insert_bytes(&self, index: usize, val: impl Into<Vec<u8>>) {
        if let Some(val_os) = OsString::from_io_vec(val.into()) {
            self.insert(index, val_os);
        }
    }

    pub fn remove_bytes(&self, index: usize) -> Option<Vec<u8>> {
        self.remove(index).and_then(OsString::into_io_vec)
    }

    // Plain lua table conversion

    #[doc(hidden)]
    #[allow(clippy::missing_errors_doc)]
    pub fn into_plain_lua_table(&self, lua: Lua) -> LuaResult<LuaTable> {
        let all = self.all_bytes();
        let tab = lua.create_table_with_capacity(all.len(), 0)?;

        for val in all {
            let val = lua.create_string(val)?;
            tab.push(val)?;
        }

        Ok(tab)
    }
}

// Iterator implementations

impl IntoIterator for ProcessArgs {
    type Item = OsString;
    type IntoIter = std::vec::IntoIter<OsString>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self.inner.lock().unwrap();
        inner.values.clone().into_iter()
    }
}

impl<S: Into<OsString>> FromIterator<S> for ProcessArgs {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(iter.into_iter().map(Into::into).collect())),
        }
    }
}

impl<S: Into<OsString>> Extend<S> for ProcessArgs {
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        let mut inner = self.inner.lock().unwrap();
        inner.values.extend(iter.into_iter().map(Into::into));
    }
}

// Lua implementations

impl FromLua for ProcessArgs {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            Ok(Self::from_iter([] as [OsString; 0]))
        } else if let LuaValue::Boolean(true) = value {
            Ok(Self::current())
        } else if let Some(u) = value.as_userdata().and_then(|u| u.borrow::<Self>().ok()) {
            Ok(u.clone())
        } else if let LuaValue::Table(arr) = value {
            let mut args = Vec::new();
            for pair in arr.pairs::<LuaValue, LuaValue>() {
                let val_res = pair.map(|p| p.1.clone());
                let val = super::lua_value_to_os_string(val_res, "ProcessArgs")?;

                super::validate_os_value(&val)?;

                args.push(val);
            }
            Ok(Self::from_iter(args))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: String::from("ProcessArgs"),
                message: Some(format!(
                    "Invalid type for process args - expected table or nil, got '{}'",
                    value.type_name()
                )),
            })
        }
    }
}

impl LuaUserData for ProcessArgs {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, (): ()| Ok(this.len()));
        methods.add_meta_method(LuaMetaMethod::Index, |_, this, index: usize| {
            if index == 0 {
                Ok(None)
            } else {
                Ok(this.get(index - 1))
            }
        });
        methods.add_meta_method(LuaMetaMethod::NewIndex, |_, _, (): ()| {
            Err::<(), _>(LuaError::runtime("ProcessArgs is read-only"))
        });
        methods.add_meta_method(LuaMetaMethod::Iter, |lua, this, (): ()| {
            let mut vars = this
                .clone()
                .into_iter()
                .filter_map(OsStringBytes::into_io_vec)
                .enumerate();
            lua.create_function_mut(move |lua, (): ()| match vars.next() {
                None => Ok((LuaValue::Nil, LuaValue::Nil)),
                Some((index, value)) => Ok((
                    LuaValue::Integer(index as i64),
                    LuaValue::String(lua.create_string(value)?),
                )),
            })
        });
    }
}
