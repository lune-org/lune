use crate::{library::StandardLibrary, luaurc::RequireAlias};
use mlua::prelude::*;
use mlua_luau_scheduler::{IntoLuaThread, LuaSchedulerExt};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::{
    fs,
    sync::{
        broadcast::{self, Sender},
        Mutex,
    },
};

use super::RequireError;

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
    /**

    # Errors

    - when `RequireContext::init` is called more than once on the same `Lua` instance

     */
    pub fn init(lua: &Lua) -> Result<(), RequireError> {
        if lua.set_app_data(RequireContextData::default()).is_some() {
            Err(RequireError::RequireContextInitCalledTwice)
        } else {
            Ok(())
        }
    }

    pub(crate) fn std_exists(lua: &Lua, alias: &str) -> Result<bool, RequireError> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

        Ok(data_ref.std.contains_key(alias))
    }

    pub(crate) fn require_std(
        lua: &Lua,
        require_alias: RequireAlias,
    ) -> Result<LuaMultiValue<'_>, RequireError> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

        if let Some(cached) = data_ref.std_cache.get(&require_alias) {
            let multi_vec = lua.registry_value::<Vec<LuaValue>>(cached)?;

            return Ok(LuaMultiValue::from_vec(multi_vec));
        }

        let libraries = data_ref.std.get(&require_alias.alias.as_str()).ok_or(
            RequireError::InvalidStdAlias(require_alias.alias.to_string()),
        )?;

        let std =
            libraries
                .get(require_alias.path.as_str())
                .ok_or(RequireError::StdMemberNotFound(
                    require_alias.path.to_string(),
                    require_alias.alias.to_string(),
                ))?;

        let multi = std.module(lua)?;
        let mutli_clone = multi.clone();
        let multi_reg = lua.create_registry_value(mutli_clone.into_vec())?;

        drop(data_ref);

        let mut data = lua
            .app_data_mut::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

        data.std_cache.insert(require_alias, multi_reg);

        Ok(multi)
    }

    async fn wait_for_pending<'lua>(
        lua: &'lua Lua,
        path_abs: &'_ PathBuf,
    ) -> Result<(), RequireError> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

        let pending = data_ref.pending.try_lock()?;

        if let Some(sender) = pending.get(path_abs) {
            let mut receiver = sender.subscribe();

            // unlock mutex before using async
            drop(pending);

            receiver.recv().await?;
        }

        Ok(())
    }

    async fn from_cache<'lua>(
        lua: &'lua Lua,
        path_abs: &'_ PathBuf,
    ) -> Result<Option<LuaMultiValue<'lua>>, RequireError> {
        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

        let cache = data_ref.cache.lock().await;

        match cache.get(path_abs) {
            Some(cached) => {
                let multi_vec = lua.registry_value::<Vec<LuaValue>>(cached)?;

                Ok(Some(LuaMultiValue::from_vec(multi_vec)))
            }
            None => Ok(None),
        }
    }

    pub(crate) async fn require(
        lua: &Lua,
        path_rel: PathBuf,
        path_abs: PathBuf,
    ) -> Result<LuaMultiValue, RequireError> {
        Self::wait_for_pending(lua, &path_abs).await?;

        if let Some(cached) = Self::from_cache(lua, &path_abs).await? {
            return Ok(cached);
        }

        // create a broadcast channel
        {
            let data_ref = lua
                .app_data_ref::<RequireContextData>()
                .ok_or(RequireError::RequireContextNotFound)?;

            let broadcast_tx = broadcast::Sender::new(1);

            {
                let mut pending = data_ref.pending.try_lock()?;
                pending.insert(path_abs.clone(), broadcast_tx);
            }
        }

        if !fs::try_exists(&path_abs).await? {
            return Err(RequireError::InvalidRequire(
                path_rel.to_string_lossy().to_string(),
            ));
        }

        let content = fs::read_to_string(&path_abs).await?;
        let thread = lua
            .load(&content)
            .set_name(path_abs.to_string_lossy())
            .into_lua_thread(lua)?;

        let thread_id = lua.push_thread_back(thread, ())?;
        lua.track_thread(thread_id);
        lua.wait_for_thread(thread_id).await;

        let multi = lua
            .get_thread_result(thread_id)
            .ok_or(RequireError::ThreadReturnedNone)??;

        let multi_reg = lua.create_registry_value(multi.into_vec())?;

        let data_ref = lua
            .app_data_ref::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

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

        match Self::from_cache(lua, &path_abs).await? {
            Some(cached) => Ok(cached),
            None => Err(RequireError::CacheNotFound(
                path_rel.to_string_lossy().to_string(),
            )),
        }
    }

    /**

    add a standard library into the require function

    # Example

    ```rs
    inject_std(lua, "lune", LuneStandardLibrary::Task)?;
    ```

    ```luau
    -- luau
    local task = require("@lune/task")
    ```

    # Errors

    - when `RequireStorage::init` isn't called

     */
    pub fn inject_std(
        lua: &Lua,
        alias: &'static str,
        std: impl StandardLibrary + 'static,
    ) -> Result<(), RequireError> {
        let mut data = lua
            .app_data_mut::<RequireContextData>()
            .ok_or(RequireError::RequireContextNotFound)?;

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
