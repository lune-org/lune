use std::{
    env::{self, current_dir},
    io,
    path::PathBuf,
};

use mlua::prelude::*;
use tokio::fs;

use crate::utils::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    // Preserve original require behavior if we have a special env var set,
    // returning an empty table since there are no globals to overwrite
    if env::var_os("LUAU_PWD_REQUIRE").is_some() {
        return TableBuilder::new(lua)?.build_readonly();
    }
    // Store the current pwd, and make helper functions for path conversions
    let require_pwd = current_dir()?.to_string_lossy().to_string();
    let require_info: LuaFunction = lua.named_registry_value("dbg.info")?;
    let require_error: LuaFunction = lua.named_registry_value("error")?;
    let require_get_abs_rel_paths = lua
        .create_function(
            |_, (require_pwd, require_source, require_path): (String, String, String)| {
                let mut path_relative_to_pwd = PathBuf::from(
                    &require_source
                        .trim_start_matches("[string \"")
                        .trim_end_matches("\"]"),
                )
                .parent()
                .unwrap()
                .join(require_path);
                // Try to normalize and resolve relative path segments such as './' and '../'
                if let Ok(canonicalized) =
                    path_relative_to_pwd.with_extension("luau").canonicalize()
                {
                    path_relative_to_pwd = canonicalized;
                }
                if let Ok(canonicalized) = path_relative_to_pwd.with_extension("lua").canonicalize()
                {
                    path_relative_to_pwd = canonicalized;
                }
                let absolute = path_relative_to_pwd.to_string_lossy().to_string();
                let relative = absolute.trim_start_matches(&require_pwd).to_string();
                Ok((absolute, relative))
            },
        )?
        .bind(require_pwd)?;
    /*
        We need to get the source file where require was
        called to be able to do path-relative requires,
        so we make a small wrapper to do that here, this
        will then call our actual async require function

        This must be done in lua because due to how our
        scheduler works mlua can not preserve debug info
    */
    let require_env = TableBuilder::new(lua)?
        .with_value("loaded", lua.create_table()?)?
        .with_value("cache", lua.create_table()?)?
        .with_value("info", require_info)?
        .with_value("error", require_error)?
        .with_value("paths", require_get_abs_rel_paths)?
        .with_async_function("load", load_file)?
        .build_readonly()?;
    let require_fn_lua = lua
        .load(
            r#"
            local source = info(2, "s")
            local absolute, relative = paths(source, ...)
            if loaded[absolute] ~= true then
                local first, second = load(absolute, relative)
                if first == nil or second ~= nil then
                    error("Module did not return exactly one value")
                end
                loaded[absolute] = true
                cache[absolute] = first
                return first
            else
                return cache[absolute]
            end
            "#,
        )
        .set_name("require")?
        .set_environment(require_env)?
        .into_function()?;
    TableBuilder::new(lua)?
        .with_value("require", require_fn_lua)?
        .build_readonly()
}

async fn load_file(
    lua: &Lua,
    (path_absolute, path_relative): (String, String),
) -> LuaResult<LuaValue> {
    // Try to read the wanted file, note that we use bytes instead of reading
    // to a string since lua scripts are not necessarily valid utf-8 strings
    match fs::read(&path_absolute).await {
        Ok(contents) => lua.load(&contents).set_name(path_relative)?.eval(),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Err(LuaError::RuntimeError(format!(
                "No lua module exists at the path '{path_relative}'"
            ))),
            _ => Err(LuaError::external(e)),
        },
    }
}
