use std::{collections::HashMap, env, path::PathBuf, sync::Arc};

use mlua::prelude::*;
use tokio::{fs, sync::Mutex as AsyncMutex};

use crate::lune::{
    builtins::LuneBuiltin,
    scheduler::{IntoLuaOwnedThread, Scheduler, SchedulerThreadId},
};

const REGISTRY_KEY: &str = "RequireContext";

#[derive(Debug, Clone)]
pub(super) struct RequireContext {
    use_cwd_relative_paths: bool,
    working_directory: PathBuf,
    cache_builtins: Arc<AsyncMutex<HashMap<LuneBuiltin, LuaResult<LuaRegistryKey>>>>,
    cache_results: Arc<AsyncMutex<HashMap<PathBuf, LuaResult<LuaRegistryKey>>>>,
    cache_pending: Arc<AsyncMutex<HashMap<PathBuf, SchedulerThreadId>>>,
}

impl RequireContext {
    /**
        Creates a new require context for the given [`Lua`] struct.

        Note that this require context is global and only one require
        context should be created per [`Lua`] struct, creating more
        than one context may lead to undefined require-behavior.
    */
    pub fn new() -> Self {
        let cwd = env::current_dir().expect("Failed to get current working directory");
        Self {
            // FUTURE: We could load some kind of config or env var
            // to check if we should be using cwd-relative paths
            use_cwd_relative_paths: false,
            working_directory: cwd,
            cache_builtins: Arc::new(AsyncMutex::new(HashMap::new())),
            cache_results: Arc::new(AsyncMutex::new(HashMap::new())),
            cache_pending: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    /**
        If `require` should use cwd-relative paths or not.
    */
    pub fn use_cwd_relative_paths(&self) -> bool {
        self.use_cwd_relative_paths
    }

    /**
        Transforms the path into an absolute path.

        If the given path is already an absolute path, this
        will only resolve path segments such as `./`, `../`, ...

        If the given path is not absolute, it first gets transformed into an
        absolute path by prepending the path to the current working directory.
    */
    fn abs_path(&self, path: impl AsRef<str>) -> PathBuf {
        let path = path_clean::clean(path.as_ref());
        if path.is_absolute() {
            path
        } else {
            self.working_directory.join(path)
        }
    }

    /**
        Checks if the given path has a cached require result.

        The cache uses absolute paths, so any given relative
        path will first be transformed into an absolute path.
    */
    pub fn is_cached(&self, path: impl AsRef<str>) -> LuaResult<bool> {
        let path = self.abs_path(path);
        let is_cached = self
            .cache_results
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .contains_key(&path);
        Ok(is_cached)
    }

    /**
        Checks if the given path is currently being used in `require`.

        The cache uses absolute paths, so any given relative
        path will first be transformed into an absolute path.
    */
    pub fn is_pending(&self, path: impl AsRef<str>) -> LuaResult<bool> {
        let path = self.abs_path(path);
        let is_pending = self
            .cache_pending
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .contains_key(&path);
        Ok(is_pending)
    }

    /**
        Gets the resulting value from the require cache.

        Will panic if the path has not been cached, use [`is_cached`] first.

        The cache uses absolute paths, so any given relative
        path will first be transformed into an absolute path.
    */
    pub fn get_from_cache<'lua>(
        &self,
        lua: &'lua Lua,
        path: impl AsRef<str>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let path = self.abs_path(path);

        let results = self
            .cache_results
            .try_lock()
            .expect("RequireContext may not be used from multiple threads");

        let cached = results
            .get(&path)
            .expect("Path does not exist in results cache");
        match cached {
            Err(e) => Err(e.clone()),
            Ok(key) => {
                let multi_vec = lua
                    .registry_value::<Vec<LuaValue>>(key)
                    .expect("Missing require result in lua registry");
                Ok(LuaMultiValue::from_vec(multi_vec))
            }
        }
    }

    /**
        Waits for the resulting value from the require cache.

        Will panic if the path has not been cached, use [`is_cached`] first.

        The cache uses absolute paths, so any given relative
        path will first be transformed into an absolute path.
    */
    pub async fn wait_for_cache<'lua>(
        &self,
        lua: &'lua Lua,
        path: impl AsRef<str>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let path = self.abs_path(path);
        let sched = lua
            .app_data_ref::<&Scheduler>()
            .expect("Lua struct is missing scheduler");

        let thread_id = {
            let pending = self
                .cache_pending
                .try_lock()
                .expect("RequireContext may not be used from multiple threads");
            let thread_id = pending
                .get(&path)
                .expect("Path is not currently pending require");
            *thread_id
        };

        sched.wait_for_thread(thread_id).await
    }

    /**
        Loads (requires) the file at the given path.

        The cache uses absolute paths, so any given relative
        path will first be transformed into an absolute path.
    */
    pub async fn load<'lua>(
        &self,
        lua: &'lua Lua,
        path: impl AsRef<str>,
    ) -> LuaResult<LuaMultiValue<'lua>> {
        let path = self.abs_path(path);
        let sched = lua
            .app_data_ref::<&Scheduler>()
            .expect("Lua struct is missing scheduler");

        // TODO: Store any fs error in the cache, too
        let file_contents = fs::read(&path).await?;

        // TODO: Store any lua loading/parsing error in the cache, too
        // TODO: Set chunk name as file name relative to cwd
        let file_thread = lua
            .load(file_contents)
            .into_function()?
            .into_owned_lua_thread(lua)?;

        // Schedule the thread to run and store the pending thread id in the require context
        let thread_id = {
            let thread_id = sched.push_back(file_thread, ())?;
            self.cache_pending
                .try_lock()
                .expect("RequireContext may not be used from multiple threads")
                .insert(path.clone(), thread_id);
            thread_id
        };

        // Wait for the thread to finish running
        let thread_res = sched.wait_for_thread(thread_id).await;

        // Clone the result and store it in the cache, note
        // that cloning a [`LuaValue`] will still refer to
        // the same underlying lua data and indentity
        let result = match thread_res.clone() {
            Err(e) => Err(e),
            Ok(multi) => {
                let multi_vec = multi.into_vec();
                let multi_key = lua
                    .create_registry_value(multi_vec)
                    .expect("Failed to store require result in registry");
                Ok(multi_key)
            }
        };

        // NOTE: We use the async lock and not try_lock here because
        // some other thread may be wanting to insert into the require
        // cache at the same time, and that's not an actual error case
        self.cache_results.lock().await.insert(path.clone(), result);

        // Remove the pending thread id from the require context
        self.cache_pending
            .try_lock()
            .expect("RequireContext may not be used from multiple threads")
            .remove(&path)
            .expect("Pending require thread id was unexpectedly removed");

        thread_res
    }

    /**
        Loads (requires) the builtin with the given name.
    */
    pub fn load_builtin<'lua>(
        &self,
        lua: &'lua Lua,
        name: impl AsRef<str>,
    ) -> LuaResult<LuaMultiValue<'lua>>
    where
        'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
    {
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
                        .expect("Failed to store require result in registry");
                    Ok(multi_key)
                }
            },
        );

        result
    }
}

impl LuaUserData for RequireContext {}

impl<'lua> FromLua<'lua> for RequireContext {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(ctx) = ud.borrow::<RequireContext>() {
                return Ok(ctx.clone());
            }
        }
        unreachable!("RequireContext should only be used from registry")
    }
}

impl<'lua> From<&'lua Lua> for RequireContext {
    fn from(value: &'lua Lua) -> Self {
        value
            .named_registry_value(REGISTRY_KEY)
            .expect("Missing require context in lua registry")
    }
}
