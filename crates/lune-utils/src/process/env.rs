#![allow(clippy::missing_panics_doc)]

use std::{
    collections::BTreeMap,
    env::vars_os,
    ffi::{OsStr, OsString},
    sync::{Arc, Mutex},
};

use mlua::prelude::*;
use os_str_bytes::{OsStrBytes, OsStringBytes};

// Inner (shared) struct

#[derive(Debug, Default)]
struct ProcessEnvInner {
    values: BTreeMap<OsString, OsString>,
}

impl FromIterator<(OsString, OsString)> for ProcessEnvInner {
    fn from_iter<T: IntoIterator<Item = (OsString, OsString)>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

/**
    A struct that can be easily shared, stored in Lua app data,
    and that also guarantees the pairs are valid OS strings
    that can be used for process environment variables.

    Usable directly from Lua, implementing both `FromLua` and `LuaUserData`.

    Also provides convenience methods for working with the variables
    as either `OsString` or `Vec<u8>`, where using the latter implicitly
    converts to an `OsString` and fails if the conversion is not possible.
*/
#[derive(Debug, Clone)]
pub struct ProcessEnv {
    inner: Arc<Mutex<ProcessEnvInner>>,
}

impl ProcessEnv {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProcessEnvInner::default())),
        }
    }

    #[must_use]
    pub fn current() -> Self {
        Self {
            inner: Arc::new(Mutex::new(vars_os().collect())),
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
    pub fn get_all(&self) -> Vec<(OsString, OsString)> {
        let inner = self.inner.lock().unwrap();
        inner.values.clone().into_iter().collect()
    }

    #[must_use]
    pub fn get_value(&self, key: impl AsRef<OsStr>) -> Option<OsString> {
        let key = key.as_ref();

        super::validate_os_key(key).ok()?;

        let inner = self.inner.lock().unwrap();
        inner.values.get(key).cloned()
    }

    pub fn set_value(&self, key: impl Into<OsString>, val: impl Into<OsString>) {
        let key = key.into();
        let val = val.into();

        if super::validate_os_pair((&key, &val)).is_err() {
            return;
        }

        let mut inner = self.inner.lock().unwrap();
        inner.values.insert(key, val);
    }

    pub fn remove_value(&self, key: impl AsRef<OsStr>) {
        let key = key.as_ref();

        if super::validate_os_key(key).is_err() {
            return;
        }

        let mut inner = self.inner.lock().unwrap();
        inner.values.remove(key);
    }

    // Bytes wrappers

    #[must_use]
    pub fn get_all_bytes(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.get_all()
            .into_iter()
            .filter_map(|(k, v)| Some((k.into_io_vec()?, v.into_io_vec()?)))
            .collect()
    }

    #[must_use]
    pub fn get_value_bytes(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        let key = OsStr::from_io_bytes(key.as_ref())?;
        let val = self.get_value(key)?;
        val.into_io_vec()
    }

    pub fn set_value_bytes(&self, key: impl AsRef<[u8]>, val: impl Into<Vec<u8>>) {
        let key = OsStr::from_io_bytes(key.as_ref());
        let val = OsString::from_io_vec(val.into());
        if let (Some(key), Some(val)) = (key, val) {
            self.set_value(key, val);
        }
    }

    pub fn remove_value_bytes(&self, key: impl AsRef<[u8]>) {
        let key = OsStr::from_io_bytes(key.as_ref());
        if let Some(key) = key {
            self.remove_value(key);
        }
    }
}

// Iterator implementations

impl IntoIterator for ProcessEnv {
    type Item = (OsString, OsString);
    type IntoIter = std::collections::btree_map::IntoIter<OsString, OsString>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self.inner.lock().unwrap();
        inner.values.clone().into_iter()
    }
}

impl<K: Into<OsString>, V: Into<OsString>> FromIterator<(K, V)> for ProcessEnv {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(
                iter.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .filter(|(k, v)| super::validate_os_pair((k, v)).is_ok())
                    .collect(),
            )),
        }
    }
}

impl<K: Into<OsString>, V: Into<OsString>> Extend<(K, V)> for ProcessEnv {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let mut inner = self.inner.lock().unwrap();
        inner.values.extend(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .filter(|(k, v)| super::validate_os_pair((k, v)).is_ok()),
        );
    }
}

// Lua implementations

impl FromLua for ProcessEnv {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            Ok(Self::from_iter([] as [(OsString, OsString); 0]))
        } else if let LuaValue::Boolean(true) = value {
            Ok(Self::current())
        } else if let Some(u) = value.as_userdata().and_then(|u| u.borrow::<Self>().ok()) {
            Ok(u.clone())
        } else if let LuaValue::Table(arr) = value {
            let mut args = Vec::new();
            for pair in arr.pairs::<LuaValue, LuaValue>() {
                let (key_res, val_res) = match pair {
                    Ok((key, val)) => (Ok(key), Ok(val)),
                    Err(err) => (Err(err.clone()), Err(err)),
                };

                let key = super::lua_value_to_os_string(key_res, "ProcessEnv")?;
                let val = super::lua_value_to_os_string(val_res, "ProcessEnv")?;

                super::validate_os_pair((&key, &val))?;

                args.push((key, val));
            }
            Ok(Self::from_iter(args))
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: String::from("ProcessEnv"),
                message: Some(format!(
                    "Invalid type for process env - expected table or nil, got '{}'",
                    value.type_name()
                )),
            })
        }
    }
}

impl LuaUserData for ProcessEnv {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, (): ()| Ok(this.len()));
        methods.add_meta_method(LuaMetaMethod::Index, |_, this, key: LuaValue| {
            let key = super::lua_value_to_os_string(Ok(key), "OsString")?;
            Ok(this.get_value(key))
        });
        methods.add_meta_method(
            LuaMetaMethod::NewIndex,
            |_, this, (key, val): (LuaValue, Option<LuaValue>)| {
                let key = super::lua_value_to_os_string(Ok(key), "OsString")?;
                if let Some(val) = val {
                    let val = super::lua_value_to_os_string(Ok(val), "OsString")?;
                    this.set_value(key, val);
                } else {
                    this.remove_value(key);
                }
                Ok(())
            },
        );
        methods.add_meta_method(LuaMetaMethod::Iter, |lua, this, (): ()| {
            let mut vars = this
                .clone()
                .into_iter()
                .filter_map(|(key, val)| Some((key.into_io_vec()?, val.into_io_vec()?)));
            lua.create_function_mut(move |lua, (): ()| match vars.next() {
                None => Ok((LuaValue::Nil, LuaValue::Nil)),
                Some((key, val)) => Ok((
                    LuaValue::String(lua.create_string(key)?),
                    LuaValue::String(lua.create_string(val)?),
                )),
            })
        });
    }
}
