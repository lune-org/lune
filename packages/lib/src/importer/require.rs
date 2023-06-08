use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    env::current_dir,
    path::{self, PathBuf},
    sync::Arc,
};

use dunce::canonicalize;
use mlua::prelude::*;
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;

use crate::lua::{
    table::TableBuilder,
    task::{TaskScheduler, TaskSchedulerScheduleExt},
};

use super::require_waker::{RequireWakerFuture, RequireWakerState};

const REQUIRE_IMPL_LUA: &str = r#"
local source = info(1, "s")
if source == '[string "require"]' then
    source = info(2, "s")
end
load(context, source, ...)
return yield()
"#;

type RequireWakersVec<'lua> = Vec<Arc<AsyncMutex<RequireWakerState<'lua>>>>;

#[derive(Debug, Clone, Default)]
struct RequireContext<'lua> {
    // NOTE: We need to use arc here so that mlua clones
    // the reference and not the entire inner value(s)
    builtins: Arc<HashMap<String, LuaMultiValue<'lua>>>,
    cached: Arc<RefCell<HashMap<String, LuaResult<LuaMultiValue<'lua>>>>>,
    wakers: Arc<RefCell<HashMap<String, RequireWakersVec<'lua>>>>,
    locks: Arc<RefCell<HashSet<String>>>,
    pwd: String,
}

impl<'lua> RequireContext<'lua> {
    pub fn new<K, V>(lua: &'lua Lua, builtins_vec: Vec<(K, V)>) -> LuaResult<Self>
    where
        K: Into<String>,
        V: IntoLua<'lua>,
    {
        let mut pwd = current_dir()
            .expect("Failed to access current working directory")
            .to_string_lossy()
            .to_string();
        if !pwd.ends_with(path::MAIN_SEPARATOR) {
            pwd = format!("{pwd}{}", path::MAIN_SEPARATOR)
        }
        let mut builtins = HashMap::new();
        for (key, value) in builtins_vec {
            builtins.insert(key.into(), value.into_lua_multi(lua)?);
        }
        Ok(Self {
            pwd,
            builtins: Arc::new(builtins),
            ..Default::default()
        })
    }

    pub fn is_locked(&self, absolute_path: &str) -> bool {
        self.locks.borrow().contains(absolute_path)
    }

    pub fn set_locked(&self, absolute_path: &str) -> bool {
        self.locks.borrow_mut().insert(absolute_path.to_string())
    }

    pub fn set_unlocked(&self, absolute_path: &str) -> bool {
        self.locks.borrow_mut().remove(absolute_path)
    }

    pub fn try_acquire_lock_sync(&self, absolute_path: &str) -> bool {
        if self.is_locked(absolute_path) {
            false
        } else {
            self.set_locked(absolute_path);
            true
        }
    }

    pub fn set_cached(&self, absolute_path: &str, result: &LuaResult<LuaMultiValue<'lua>>) {
        self.cached
            .borrow_mut()
            .insert(absolute_path.to_string(), result.clone());
        if let Some(wakers) = self.wakers.borrow_mut().remove(absolute_path) {
            for waker in wakers {
                waker
                    .try_lock()
                    .expect("Failed to lock waker")
                    .finalize(result.clone());
            }
        }
    }

    pub fn wait_for_cache(self, absolute_path: &str) -> RequireWakerFuture<'lua> {
        let state = RequireWakerState::new();
        let fut = RequireWakerFuture::new(&state);
        self.wakers
            .borrow_mut()
            .entry(absolute_path.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::clone(&state));
        fut
    }

    pub fn get_paths(
        &self,
        require_source: String,
        require_path: String,
    ) -> LuaResult<(String, String)> {
        if require_path.starts_with('@') {
            return Ok((require_path.clone(), require_path));
        }
        let path_relative_to_pwd = PathBuf::from(
            &require_source
                .trim_start_matches("[string \"")
                .trim_end_matches("\"]"),
        )
        .parent()
        .unwrap()
        .join(&require_path);
        // Try to normalize and resolve relative path segments such as './' and '../'
        let file_path = match (
            canonicalize(path_relative_to_pwd.with_extension("luau")),
            canonicalize(path_relative_to_pwd.with_extension("lua")),
        ) {
            (Ok(luau), _) => luau,
            (_, Ok(lua)) => lua,
            _ => {
                return Err(LuaError::RuntimeError(format!(
                    "File does not exist at path '{require_path}'"
                )))
            }
        };
        let absolute = file_path.to_string_lossy().to_string();
        let relative = absolute.trim_start_matches(&self.pwd).to_string();
        Ok((absolute, relative))
    }
}

impl<'lua> LuaUserData for RequireContext<'lua> {}

fn load_builtin<'lua>(
    _lua: &'lua Lua,
    context: RequireContext<'lua>,
    module_name: String,
    _has_acquired_lock: bool,
) -> LuaResult<LuaMultiValue<'lua>> {
    match context.builtins.get(&module_name) {
        Some(module) => Ok(module.clone()),
        None => Err(LuaError::RuntimeError(format!(
            "No builtin module exists with the name '{}'",
            module_name
        ))),
    }
}

async fn load_file<'lua>(
    lua: &'lua Lua,
    context: RequireContext<'lua>,
    absolute_path: String,
    relative_path: String,
    has_acquired_lock: bool,
) -> LuaResult<LuaMultiValue<'lua>> {
    let cached = { context.cached.borrow().get(&absolute_path).cloned() };
    match cached {
        Some(cached) => cached,
        None => {
            if !has_acquired_lock {
                return context.wait_for_cache(&absolute_path).await;
            }
            // Try to read the wanted file, note that we use bytes instead of reading
            // to a string since lua scripts are not necessarily valid utf-8 strings
            let contents = fs::read(&absolute_path).await.map_err(LuaError::external)?;
            // Use a name without extensions for loading the chunk, some
            // other code assumes the require path is without extensions
            let path_relative_no_extension = relative_path
                .trim_end_matches(".lua")
                .trim_end_matches(".luau");
            // Load the file into a thread
            let loaded_func = lua
                .load(&contents)
                .set_name(path_relative_no_extension)
                .into_function()?;
            let loaded_thread = lua.create_thread(loaded_func)?;
            // Run the thread and wait for completion using the native task scheduler waker
            let task_fut = {
                let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
                let task = sched.schedule_blocking(loaded_thread, LuaMultiValue::new())?;
                sched.wait_for_task_completion(task)
            };
            // Wait for the thread to finish running, cache + return our result,
            // notify any other threads that are also waiting on this to finish
            let rets = task_fut.await;
            context.set_cached(&absolute_path, &rets);
            rets
        }
    }
}

async fn load<'lua>(
    lua: &'lua Lua,
    context: LuaUserDataRef<'lua, RequireContext<'lua>>,
    absolute_path: String,
    relative_path: String,
    has_acquired_lock: bool,
) -> LuaResult<LuaMultiValue<'lua>> {
    let result = if absolute_path == relative_path && absolute_path.starts_with('@') {
        if let Some(module_name) = absolute_path.strip_prefix("@lune/") {
            load_builtin(
                lua,
                context.clone(),
                module_name.to_string(),
                has_acquired_lock,
            )
        } else {
            // FUTURE: '@' can be used a special prefix for users to set their own
            // paths relative to a project file, similar to typescript paths config
            // https://www.typescriptlang.org/tsconfig#paths
            Err(LuaError::RuntimeError(
                "Require paths prefixed by '@' are not yet supported".to_string(),
            ))
        }
    } else {
        load_file(
            lua,
            context.clone(),
            absolute_path.to_string(),
            relative_path,
            has_acquired_lock,
        )
        .await
    };
    if has_acquired_lock {
        context.set_unlocked(&absolute_path);
    }
    result
}

pub fn create<K, V>(lua: &'static Lua, builtins: Vec<(K, V)>) -> LuaResult<LuaFunction>
where
    K: Clone + Into<String>,
    V: Clone + IntoLua<'static>,
{
    let require_context = RequireContext::new(lua, builtins)?;
    let require_yield: LuaFunction = lua.named_registry_value("co.yield")?;
    let require_info: LuaFunction = lua.named_registry_value("dbg.info")?;
    let require_print: LuaFunction = lua.named_registry_value("print")?;

    let require_env = TableBuilder::new(lua)?
        .with_value("context", require_context)?
        .with_value("yield", require_yield)?
        .with_value("info", require_info)?
        .with_value("print", require_print)?
        .with_function(
            "load",
            |lua,
             (context, require_source, require_path): (
                LuaUserDataRef<RequireContext>,
                String,
                String,
            )| {
                let (absolute_path, relative_path) =
                    context.get_paths(require_source, require_path)?;
                // NOTE: We can not acquire the lock in the async part of the require
                // load process since several requires may have happened for the
                // same path before the async load task even gets a chance to run
                let has_lock = context.try_acquire_lock_sync(&absolute_path);
                let fut = load(lua, context, absolute_path, relative_path, has_lock);
                let sched = lua
                    .app_data_ref::<&TaskScheduler>()
                    .expect("Missing task scheduler as a lua app data");
                sched.queue_async_task_inherited(lua.current_thread(), None, async {
                    let rets = fut.await?;
                    let mult = rets.into_lua_multi(lua)?;
                    Ok(Some(mult))
                })
            },
        )?
        .build_readonly()?;

    let require_fn_lua = lua
        .load(REQUIRE_IMPL_LUA)
        .set_name("require")
        .set_environment(require_env)
        .into_function()?;
    Ok(require_fn_lua)
}
