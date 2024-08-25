use std::path::PathBuf;

/**

return's the path of the script that called this function

 */
pub fn get_script_path(lua: &mlua::Lua) -> Result<PathBuf, mlua::Error> {
    let Some(debug) = lua.inspect_stack(2) else {
        return Err(mlua::Error::runtime("Failed to inspect stack"));
    };

    match debug
        .source()
        .source
        .map(|raw_source| PathBuf::from(raw_source.to_string()))
    {
        Some(script) => Ok(script),
        None => Err(mlua::Error::runtime(
            "Failed to get path of the script that called require",
        )),
    }
}

/**

return's the parent directory of the script that called this function

 */
pub fn get_parent_path(lua: &mlua::Lua) -> Result<PathBuf, mlua::Error> {
    let script = get_script_path(lua)?;

    match script.parent() {
        Some(parent) => Ok(parent.to_path_buf()),
        None => Err(mlua::Error::runtime(
            "Failed to get parent of the script that called require",
        )),
    }
}
