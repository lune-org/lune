use crate::{library::StandardLibrary, luaurc::RequireAlias};
use mlua::prelude::*;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{
    fs,
    sync::{
        broadcast::{self, Sender},
        Mutex,
    },
};

/// The private struct that's stored in mlua's app data container
#[derive(Debug, Default)]
struct RequireContextData<'a> {
    std: HashMap<&'a str, HashMap<&'a str, Box<dyn StandardLibrary>>>,
    std_cache: HashMap<RequireAlias, LuaRegistryKey>,
    cache: Arc<Mutex<HashMap<PathBuf, LuaRegistryKey>>>,
    pending: Arc<Mutex<HashMap<PathBuf, Sender<()>>>>,
}

#[derive(Debug)]
pub struct RequireContext {}

impl RequireContext {
    pub fn init(lua: &Lua) -> LuaResult<()> {
        if lua.set_app_data(RequireContextData::default()).is_some() {
            Err(LuaError::runtime("RequireContext::init got called twice"))
        } else {
            Ok(())
        }
    }

    pub fn std_exists(lua: &Lua, alias: &str) -> LuaResult<bool> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        Ok(data_ref.std.contains_key(alias))
    }

    pub fn require_std(lua: &Lua, require_alias: RequireAlias) -> LuaResult<LuaMultiValue<'_>> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        if let Some(cached) = data_ref.std_cache.get(&require_alias) {
            return cached.into_lua(lua)?.into_lua_multi(lua);
        }

        let libraries =
            data_ref
                .std
                .get(&require_alias.alias.as_str())
                .ok_or(mlua::Error::runtime(format!(
                    "Alias '{}' does not point to a built-in standard library",
                    require_alias.alias
                )))?;

        let std = libraries
            .get(require_alias.path.as_str())
            .ok_or(mlua::Error::runtime(format!(
                "Library '{}' does not point to a member of '{}' standard libraries",
                require_alias.path, require_alias.alias
            )))?;

        let multi = std.module(lua)?;
        let mutli_clone = multi.clone();
        let multi_reg = lua.create_registry_value(mutli_clone.into_vec())?;

        let mut data = lua
        .app_data_mut::<RequireContextData>()
        .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        data.std_cache.insert(require_alias, multi_reg);

        Ok(multi)
    }

    pub async fn require(
        lua: &Lua,
        path_rel: PathBuf,
        path_abs: PathBuf,
    ) -> LuaResult<LuaMultiValue> {
        // wait for module to be required
        // if its pending somewhere else
        {
            let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

            let pending = data_ref.pending.try_lock().into_lua_err()?;

            if let Some(a) = pending.get(&path_abs) {
                a.subscribe().recv().await.into_lua_err()?;
            }
        }

        // get module from cache
        // *if* its cached
        {
            let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

            let cache = data_ref.cache.lock().await;

            if let Some(cached) = cache.get(&path_abs) {
                return cached.into_lua(lua).into_lua_multi(lua);
            }
        }

        // create a broadcast channel
        {
            let data_ref = lua
        .app_data_ref::<RequireContextData>()
        .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

            let (broadcast_tx, _) = broadcast::channel(1);

            {
                let mut pending = data_ref.pending.try_lock().into_lua_err()?;
                pending.insert(path_abs.clone(), broadcast_tx);
            }
        }

        if !fs::try_exists(&path_abs).await? {
            return Err(LuaError::runtime(format!(
                "Can not require '{}' as it does not exist",
                path_rel.to_string_lossy()
            )));
        }

        let content = fs::read_to_string(&path_abs).await?;

        let multi = lua
            .load(content)
            .set_name(path_abs.to_string_lossy())
            .eval_async::<LuaMultiValue>()
            .await?;

        let mutli_clone = multi.clone();
        let multi_reg = lua.create_registry_value(mutli_clone.into_vec())?;

        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        data_ref
            .cache
            .lock()
            .await
            .insert(path_abs.clone(), multi_reg);

        let broadcast_tx = data_ref
            .pending
            .lock()
            .await
            .remove(&path_abs)
            .expect("Pending require broadcaster was unexpectedly removed");

        broadcast_tx.send(()).ok();

        Ok(multi)
    }

    pub fn inject_std(
        lua: &Lua,
        alias: &'static str,
        std: impl StandardLibrary + 'static,
    ) -> LuaResult<()> {
        let mut data = lua
            .app_data_mut::<RequireContextData>()
            .ok_or(LuaError::runtime("Couldn't find RequireContextData in app data container, make sure RequireStorage::init is called on this lua instance"))?;

        if let Some(map) = data.std.get_mut(alias) {
            map.insert(std.name(), Box::new(std));
        } else {
            let mut map: HashMap<&str, Box<dyn StandardLibrary>> = HashMap::new();

            map.insert(std.name(), Box::new(std));

            data.std.insert(alias, map);
        };

        Ok(())
    }
}
