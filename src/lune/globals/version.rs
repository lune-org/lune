use mlua::prelude::*;

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLua<'_>> {
    let luau_version_full = lua
        .globals()
        .get::<_, LuaString>("_VERSION")
        .expect("Missing _VERSION global");

    let luau_version = luau_version_full
        .to_str()?
        .strip_prefix("Luau 0.")
        .expect("_VERSION global is formatted incorrectly")
        .trim();

    if luau_version.is_empty() {
        panic!("_VERSION global is missing version number")
    }

    lua.create_string(format!(
        "Lune {lune}+{luau}",
        lune = env!("CARGO_PKG_VERSION"),
        luau = luau_version,
    ))
}
