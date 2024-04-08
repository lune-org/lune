use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSchedulerExt;
use tokio::{
    fs,
    sync::{
        broadcast::{self, Sender},
        Mutex as AsyncMutex,
    },
};

use crate::lune::{builtins::LuneBuiltin, util::paths::CWD};

/**
    Context containing cached results for all `require` operations.

    The cache uses absolute paths, so any given relative
    path will first be transformed into an absolute path.
*/
#[derive(Debug, Clone)]
pub(super) struct RequireContext {
    cache_builtins: Arc<AsyncMutex<HashMap<LuneBuiltin, LuaResult<LuaRegistryKey>>>>,
    cache_results: Arc<AsyncMutex<HashMap<PathBuf, LuaResult<LuaRegistryKey>>>>,
    cache_pending: Arc<AsyncMutex<HashMap<PathBuf, Sender<()>>>>,
}

impl RequireContext {
    /**
        Creates a new require context for the given [`Lua`] struct.

        Note that this require context is global and only one require
        context should be created per [`Lua`] struct, creating more
        than one context may lead to undefined require-behavior.
    */
    pub fn new() -> Self {
        Self {
            cache_builtins: Arc::new(AsyncMutex::new(HashMap::new())),
            cache_results: Arc::new(AsyncMutex::new(HashMap::new())),
            cache_pending: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    /**
        Resolves the given `source` and `path` into require paths
        to use, based on the current require context settings.

        This will resolve path segments such as `./`, `../`, ..., and
        if the resolved path is not an absolute path, will create an
        absolute path by prepending the current working directory.
    */
    pub fn resolve_paths(
        &self,
        source: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> LuaResult<(PathBuf, PathBuf)> {
        let path = PathBuf::from(source.as_ref())
            .parent()
            .ok_or_else(|| LuaError::runtime("Failed to get parent path of source"))?
            .join(path.as_ref());

        let rel_path = path_clean::clean(path);
        let abs_path = if rel_path.is_absolute() {
            rel_path.to_path_buf()
        } else {
            CWD.join(&rel_path)
        };

        Ok((abs_path, rel_path))
    }

    /**
        Checks if the given path has a cached require result.
    */
    pub fn is_cached(&self, abs_path: impl AsRef<Path>) -> LuaResult<bool> {
        let is_cached = self
            .cache_results
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .contains_key(abs_path.as_ref());
        Ok(is_cached)
    }

    /**
        Checks if the given path is currently being used in `require`.
    */
    pub fn is_pending(&self, abs_path: impl AsRef<Path>) -> LuaResult<bool> {
        let is_pending = self
            .cache_pending
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .contains_key(abs_path.as_ref());
        Ok(is_pending)
    }

    /**
        Gets the resulting value from the require cache.

        Will panic if the path has not been cached, use [`is_cached`] first.
    */
    pub fn get_from_cache<'lua>(
        &self,
        lua: &'lua Lua,
        abs_path: impl AsRef<Path>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let results = self
            .cache_results
            .try_lock()
            .expect("RequireContext may not be used from multiple threads");

        let cached = results
            .get(abs_path.as_ref())
            .expect("Path does not exist in results cache");
        match cached {
            Err(e) => Err(e.clone()),
            Ok(k) => {
                let multi_vec = lua
                    .registry_value::<Vec<LuaValue>>(k)
                    .expect("Missing require result in lua registry");
                Ok(LuaMultiValue::from_vec(multi_vec))
            }
        }
    }

    /**
        Waits for the resulting value from the require cache.

        Will panic if the path has not been cached, use [`is_cached`] first.
    */
    pub async fn wait_for_cache<'lua>(
        &self,
        lua: &'lua Lua,
        abs_path: impl AsRef<Path>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let mut thread_recv = {
            let pending = self
                .cache_pending
                .try_lock()
                .expect("RequireContext may not be used from multiple threads");
            let thread_id = pending
                .get(abs_path.as_ref())
                .expect("Path is not currently pending require");
            thread_id.subscribe()
        };

        thread_recv.recv().await.into_lua_err()?;

        self.get_from_cache(lua, abs_path.as_ref())
    }

    async fn load<'lua>(
        &self,
        lua: &'lua Lua,
        abs_path: impl AsRef<Path>,
        rel_path: impl AsRef<Path>,
    ) -> LuaResult<LuaRegistryKey> {
        let abs_path = abs_path.as_ref();
        let rel_path = rel_path.as_ref();

        // Read the file at the given path, try to parse and
        // load it into a new lua thread that we can schedule
        let file_contents = fs::read(&abs_path).await?;
        let file_thread = lua
            .load(file_contents)
            .set_name(rel_path.to_string_lossy().to_string());

        // Schedule the thread to run, wait for it to finish running
        let thread_id = lua.push_thread_back(file_thread, ())?;
        lua.track_thread(thread_id);
        lua.wait_for_thread(thread_id).await;
        let thread_res = lua.get_thread_result(thread_id).unwrap();

        // Return the result of the thread, storing any lua value(s) in the registry
        match thread_res {
            Err(e) => Err(e),
            Ok(v) => {
                let multi_vec = v.into_vec();
                let multi_key = lua
                    .create_registry_value(multi_vec)
                    .expect("Failed to store require result in registry - out of memory");
                Ok(multi_key)
            }
        }
    }

    /**
        Loads (requires) the file at the given path.
    */
    pub async fn load_with_caching<'lua>(
        &self,
        lua: &'lua Lua,
        abs_path: impl AsRef<Path>,
        rel_path: impl AsRef<Path>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let abs_path = abs_path.as_ref();
        let rel_path = rel_path.as_ref();

        // Set this abs path as currently pending
        let (broadcast_tx, _) = broadcast::channel(1);
        self.cache_pending
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .insert(abs_path.to_path_buf(), broadcast_tx);

        // Try to load at this abs path
        let load_res = self.load(lua, abs_path, rel_path).await;
        let load_val = match &load_res {
            Err(e) => Err(e.clone()),
            Ok(k) => {
                let multi_vec = lua
                    .registry_value::<Vec<LuaValue>>(k)
                    .expect("Failed to fetch require result from registry");
                Ok(LuaMultiValue::from_vec(multi_vec))
            }
        };

        // NOTE: We use the async lock and not try_lock here because
        // some other thread may be wanting to insert into the require
        // cache at the same time, and that's not an actual error case
        self.cache_results
            .lock()
            .await
            .insert(abs_path.to_path_buf(), load_res);

        // Remove the pending thread id from the require context,
        // broadcast a message to let any listeners know that this
        // path has now finished the require process and is cached
        let broadcast_tx = self
            .cache_pending
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .remove(abs_path)
            .expect("Pending require broadcaster was unexpectedly removed");
        broadcast_tx.send(()).ok();

        load_val
    }

    /**
        Loads (requires) the builtin with the given name.
    */
    pub fn load_builtin<'lua>(
        &self,
        lua: &'lua Lua,
        name: impl AsRef<str>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let builtin: LuneBuiltin = match name.as_ref().parse() {
            Err(e) => return Err(LuaError::runtime(e)),
            Ok(b) => b,
        };

        let mut cache = self
            .cache_builtins
            .try_lock()
            .expect("RequireContext may not be used from multiple threads");

        if let Some(res) = cache.get(&builtin) {
            return match res {
                Err(e) => return Err(e.clone()),
                Ok(key) => {
                    let multi_vec = lua
                        .registry_value::<Vec<LuaValue>>(key)
                        .expect("Missing builtin result in lua registry");
                    Ok(LuaMultiValue::from_vec(multi_vec))
                }
            };
        };

        let result = builtin.create(lua);

        cache.insert(
            builtin,
            match result.clone() {
                Err(e) => Err(e),
                Ok(multi) => {
                    let multi_vec = multi.into_vec();
                    let multi_key = lua
                        .create_registry_value(multi_vec)
                        .expect("Failed to store require result in registry - out of memory");
                    Ok(multi_key)
                }
            },
        );

        result
    }
}
