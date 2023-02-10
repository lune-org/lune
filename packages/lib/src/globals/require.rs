use std::{
    env::{self, current_dir},
    path::PathBuf,
    sync::Arc,
};

use mlua::prelude::*;
use os_str_bytes::{OsStrBytes, RawOsStr};

use crate::utils::table::TableBuilder;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    let require: LuaFunction = lua.globals().raw_get("require")?;
    // Preserve original require behavior if we have a special env var set
    if env::var_os("LUAU_PWD_REQUIRE").is_some() {
        return TableBuilder::new(lua)?
            .with_value("require", require)?
            .build_readonly();
    }
    /*
      Store the current working directory so that we can use it later
      and remove it from require paths in error messages, showing
      absolute paths is bad ux and we should try to avoid it

      Throughout this function we also take extra care to not perform any lossy
      conversion and use os strings instead of Rust's utf-8 checked strings,
      just in case someone out there uses luau with non-utf8 string requires
    */
    let pwd = lua.create_string(&current_dir()?.to_raw_bytes())?;
    lua.set_named_registry_value("require_pwd", pwd)?;
    // Fetch the debug info function and store it in the registry
    // - we will use it to fetch the current scripts file name
    let debug: LuaTable = lua.globals().raw_get("debug")?;
    let info: LuaFunction = debug.raw_get("info")?;
    lua.set_named_registry_value("require_getinfo", info)?;
    // Store the original require function in the registry
    lua.set_named_registry_value("require_original", require)?;
    /*
      Create a new function that fetches the file name from the current thread,
      sets the luau module lookup path to be the exact script we are looking
      for, and then runs the original require function with the wanted path
    */
    let new_require = lua.create_function(|lua, require_path: LuaString| {
        let require_pwd: LuaString = lua.named_registry_value("require_pwd")?;
        let require_original: LuaFunction = lua.named_registry_value("require_original")?;
        let require_getinfo: LuaFunction = lua.named_registry_value("require_getinfo")?;
        let require_source: LuaString = require_getinfo.call((2, "s"))?;
        /*
          Combine the require caller source with the wanted path
          string to get a final path relative to pwd - it is definitely
          relative to pwd because Lune will only load files relative to pwd
        */
        let raw_pwd_str = RawOsStr::assert_from_raw_bytes(require_pwd.as_bytes());
        let raw_source = RawOsStr::assert_from_raw_bytes(require_source.as_bytes());
        let raw_path = RawOsStr::assert_from_raw_bytes(require_path.as_bytes());
        let mut path_relative_to_pwd = PathBuf::from(&raw_source.to_os_str())
            .parent()
            .unwrap()
            .join(raw_path.to_os_str());
        // Try to normalize and resolve relative path segments such as './' and '../'
        if let Ok(canonicalized) = path_relative_to_pwd.with_extension("luau").canonicalize() {
            path_relative_to_pwd = canonicalized.with_extension("");
        }
        if let Ok(canonicalized) = path_relative_to_pwd.with_extension("lua").canonicalize() {
            path_relative_to_pwd = canonicalized.with_extension("");
        }
        if let Ok(stripped) = path_relative_to_pwd.strip_prefix(&raw_pwd_str.to_os_str()) {
            path_relative_to_pwd = stripped.to_path_buf();
        }
        // Create a lossless lua string from the pathbuf and finally call require
        let raw_path_str = RawOsStr::new(path_relative_to_pwd.as_os_str());
        let lua_path_str = lua.create_string(raw_path_str.as_raw_bytes());
        // If the require call errors then we should also replace
        // the path in the error message to improve user experience
        let result: LuaResult<_> = require_original.call::<_, LuaValue>(lua_path_str);
        match result {
            Err(LuaError::CallbackError { traceback, cause }) => {
                let before = format!(
                    "runtime error: cannot find '{}'",
                    path_relative_to_pwd.to_str().unwrap()
                );
                let after = format!(
                    "Invalid require path '{}' ({})",
                    require_path.to_str().unwrap(),
                    path_relative_to_pwd.to_str().unwrap()
                );
                let cause = Arc::new(LuaError::RuntimeError(
                    cause.to_string().replace(&before, &after),
                ));
                Err(LuaError::CallbackError { traceback, cause })
            }
            Err(e) => Err(e),
            Ok(result) => Ok(result),
        }
    })?;
    // Override the original require global with our monkey-patched one
    TableBuilder::new(lua)?
        .with_value("require", new_require)?
        .build_readonly()
}
