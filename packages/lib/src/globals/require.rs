use std::{
    cell::RefCell,
    collections::HashMap,
    env::current_dir,
    path::{self, PathBuf},
};

use dunce::canonicalize;
use mlua::prelude::*;
use tokio::{fs, sync::oneshot};

use crate::lua::{
    table::TableBuilder,
    task::{TaskScheduler, TaskSchedulerScheduleExt},
};

const REQUIRE_IMPL_LUA: &str = r#"
local source = info(1, "s")
if source == '[string "require"]' then
    source = info(2, "s")
end
local absolute, relative = importer:paths(source, ...)
return importer:load(thread(), absolute, relative)
"#;

#[derive(Debug, Clone, Default)]
struct Importer<'lua> {
    builtins: HashMap<String, LuaMultiValue<'lua>>,
    cached: RefCell<HashMap<String, LuaResult<LuaMultiValue<'lua>>>>,
    pwd: String,
}

impl<'lua> Importer<'lua> {
    pub fn new() -> Self {
        let mut pwd = current_dir()
            .expect("Failed to access current working directory")
            .to_string_lossy()
            .to_string();
        if !pwd.ends_with(path::MAIN_SEPARATOR) {
            pwd = format!("{pwd}{}", path::MAIN_SEPARATOR)
        }
        Self {
            pwd,
            ..Default::default()
        }
    }

    fn paths(&self, require_source: String, require_path: String) -> LuaResult<(String, String)> {
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

    fn load_builtin(&self, module_name: &str) -> LuaResult<LuaMultiValue> {
        match self.builtins.get(module_name) {
            Some(module) => Ok(module.clone()),
            None => Err(LuaError::RuntimeError(format!(
                "No builtin module exists with the name '{}'",
                module_name
            ))),
        }
    }

    async fn load_file(
        &self,
        lua: &'lua Lua,
        absolute_path: String,
        relative_path: String,
    ) -> LuaResult<LuaMultiValue> {
        let cached = { self.cached.borrow().get(&absolute_path).cloned() };
        match cached {
            Some(cached) => cached,
            None => {
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
                    .set_name(path_relative_no_extension)?
                    .into_function()?;
                let loaded_thread = lua.create_thread(loaded_func)?;
                // Run the thread and provide a channel that will
                // then get its result received when it finishes
                let (tx, rx) = oneshot::channel();
                {
                    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
                    let task = sched.schedule_blocking(loaded_thread, LuaMultiValue::new())?;
                    sched.set_task_result_sender(task, tx);
                }
                // Wait for the thread to finish running, cache + return our result
                let rets = rx.await.expect("Sender was dropped during require");
                self.cached.borrow_mut().insert(absolute_path, rets.clone());
                rets
            }
        }
    }

    async fn load(
        &self,
        lua: &'lua Lua,
        absolute_path: String,
        relative_path: String,
    ) -> LuaResult<LuaMultiValue> {
        if absolute_path == relative_path && absolute_path.starts_with('@') {
            if let Some(module_name) = absolute_path.strip_prefix("@lune/") {
                self.load_builtin(module_name)
            } else {
                Err(LuaError::RuntimeError(
                    "Require paths prefixed by '@' are not yet supported".to_string(),
                ))
            }
        } else {
            self.load_file(lua, absolute_path, relative_path).await
        }
    }
}

impl<'i> LuaUserData for Importer<'i> {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "paths",
            |_, this, (require_source, require_path): (String, String)| {
                this.paths(require_source, require_path)
            },
        );
        methods.add_method(
            "load",
            |lua, this, (thread, absolute_path, relative_path): (LuaThread, String, String)| {
                // TODO: Make this work
                // this.load(lua, absolute_path, relative_path)
                Ok(())
            },
        );
    }
}

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let require_importer = Importer::new();
    let require_thread: LuaFunction = lua.named_registry_value("co.thread")?;
    let require_info: LuaFunction = lua.named_registry_value("dbg.info")?;
    let require_env = TableBuilder::new(lua)?
        .with_value("importer", require_importer)?
        .with_value("thread", require_thread)?
        .with_value("info", require_info)?
        .build_readonly()?;

    let require_fn_lua = lua
        .load(REQUIRE_IMPL_LUA)
        .set_name("require")?
        .set_environment(require_env)?
        .into_function()?;

    TableBuilder::new(lua)?
        .with_value("require", require_fn_lua)?
        .build_readonly()
}
