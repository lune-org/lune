use mlua::prelude::*;

struct UnsafeLibrary(bool);

/**
    Override unsafe library allowance
*/
pub fn set_unsafe_library_enabled(lua: &Lua, enabled: bool) {
    lua.set_app_data(UnsafeLibrary(enabled));
}

/**
    Returns whether unsafe libraries are allowed

    # Panics

    Panic if `UnsafeLib` app data doesn't exist.
*/
#[must_use]
pub fn get_unsafe_library_enabled(lua: &Lua) -> bool {
    lua.app_data_ref::<UnsafeLibrary>().unwrap().0
}
