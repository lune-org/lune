use mlua::prelude::*;

use lune_utils::get_version_string;

struct Version(String);

impl LuaUserData for Version {}

pub fn create(lua: &Lua) -> LuaResult<LuaValue> {
    let v = match lua.app_data_ref::<Version>() {
        Some(v) => v.0.to_string(),
        None => env!("CARGO_PKG_VERSION").to_string(),
    };
    let s = get_version_string(v);
    lua.create_string(s)?.into_lua(lua)
}

/**
    Overrides the version string to be used by the `_VERSION` global.

    The global will be a string in the format `Lune x.y.z+luau`,
    where `x.y.z` is the string passed to this function.

    The version string passed should be the version of the Lune runtime,
    obtained from `env!("CARGO_PKG_VERSION")` or a similar mechanism.

    # Panics

    Panics if the version string is empty or contains invalid characters.
*/
pub fn set_global_version(lua: &Lua, version: impl Into<String>) {
    let v = version.into();
    let _ = get_version_string(&v); // Validate version string
    lua.set_app_data(Version(v));
}
