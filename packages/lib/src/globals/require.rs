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
local absolute, relative = paths(context, source, ...)
return load(context, absolute, relative)
"#;

#[derive(Debug, Clone, Default)]
struct RequireContext<'lua> {
    builtins: HashMap<String, LuaMultiValue<'lua>>,
    cached: RefCell<HashMap<String, LuaResult<LuaMultiValue<'lua>>>>,
    pwd: String,
}

impl<'lua> RequireContext<'lua> {
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
}

impl<'lua> LuaUserData for RequireContext<'lua> {}

fn paths(
    context: RequireContext,
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
    let relative = absolute.trim_start_matches(&context.pwd).to_string();
    Ok((absolute, relative))
}

fn load_builtin<'lua>(
    _lua: &'lua Lua,
    context: RequireContext<'lua>,
    module_name: String,
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
) -> LuaResult<LuaMultiValue<'lua>> {
    let cached = { context.cached.borrow().get(&absolute_path).cloned() };
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
            // FIXME: This waits indefinitely for nested requires for some reason
            let rets = rx.await.expect("Sender was dropped during require");
            context
                .cached
                .borrow_mut()
                .insert(absolute_path, rets.clone());
            rets
        }
    }
}

async fn load<'lua>(
    lua: &'lua Lua,
    context: RequireContext<'lua>,
    absolute_path: String,
    relative_path: String,
) -> LuaResult<LuaMultiValue<'lua>> {
    if absolute_path == relative_path && absolute_path.starts_with('@') {
        if let Some(module_name) = absolute_path.strip_prefix("@lune/") {
            load_builtin(lua, context, module_name.to_string())
        } else {
            Err(LuaError::RuntimeError(
                "Require paths prefixed by '@' are not yet supported".to_string(),
            ))
        }
    } else {
        load_file(lua, context, absolute_path, relative_path).await
    }
}

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    let require_context = RequireContext::new();
    let require_print: LuaFunction = lua.named_registry_value("print")?;
    let require_info: LuaFunction = lua.named_registry_value("dbg.info")?;

    let require_env = TableBuilder::new(lua)?
        .with_value("context", require_context)?
        .with_value("print", require_print)?
        .with_value("info", require_info)?
        .with_function("paths", |_, (context, require_source, require_path)| {
            paths(context, require_source, require_path)
        })?
        .with_async_function("load", |lua, (context, require_source, require_path)| {
            load(lua, context, require_source, require_path)
        })?
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
