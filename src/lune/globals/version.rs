use mlua::prelude::*;

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    let lune_version = format!("Lune {}", env!("CARGO_PKG_VERSION"));
    
    let luau_version_full = lua
        .globals()
        .get::<_, LuaString>("_VERSION")
        .expect("Missing _VERSION global");
    let luau_version_str = luau_version_full
        .to_str()
        .context("Invalid utf8 found in _VERSION global")?;

    // If this function runs more than once, we
    // may get an already formatted lune version.
    if luau_version_str.starts_with(lune_version.as_str()) {
        return Ok(luau_version_full);
    }

    // Luau version is expected to be in the format "Luau 0.x" and sometimes "Luau 0.x.y"
    if !luau_version_str.starts_with("Luau 0.") {
        panic!("_VERSION global is formatted incorrectly\nGot: '{luau_version_str}'")
    }
    let luau_version = luau_version_str.strip_prefix("Luau 0.").unwrap().trim();

    // We make some guarantees about the format of the _VERSION global,
    // so make sure that the luau version also follows those rules.
    if luau_version.is_empty() {
        panic!("_VERSION global is missing version number\nGot: '{luau_version_str}'")
    } else if !luau_version.chars().all(is_valid_version_char) {
        panic!("_VERSION global contains invalid characters\nGot: '{luau_version_str}'")
    }

    lua.create_string(format!("{lune_version}+{luau_version}"))
}

fn is_valid_version_char(c: char) -> bool {
    matches!(c, '0'..='9' | '.')
}
