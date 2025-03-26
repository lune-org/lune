use std::sync::Arc;

use mlua::prelude::*;
use once_cell::sync::Lazy;
use semver::Version;

static LUAU_VERSION: Lazy<Arc<String>> = Lazy::new(create_luau_version_string);

/**
    Returns a Lune version string, in the format `Lune x.y.z+luau`.

    The version string passed should be the version of the Lune runtime,
    obtained from `env!("CARGO_PKG_VERSION")` or a similar mechanism.

    # Panics

    Panics if the version string is empty or contains invalid characters.
*/
#[must_use]
pub fn get_version_string(lune_version: impl AsRef<str>) -> String {
    let lune_version = lune_version.as_ref();

    assert!(!lune_version.is_empty(), "Lune version string is empty");
    match Version::parse(lune_version) {
        Ok(semver) => format!("Lune {semver}+{}", *LUAU_VERSION),
        Err(e) => panic!("Lune version string is not valid semver: {e}"),
    }
}

fn create_luau_version_string() -> Arc<String> {
    // Extract the current Luau version from a fresh Lua state / VM that can't be accessed externally.
    let luau_version_full = {
        let temp_lua = Lua::new();

        let luau_version_full = temp_lua
            .globals()
            .get::<_, LuaString>("_VERSION")
            .expect("Missing _VERSION global");

        luau_version_full
            .to_str()
            .context("Invalid utf8 found in _VERSION global")
            .expect("Expected _VERSION global to be a string")
            .to_string()
    };

    // Luau version is expected to be in the format "Luau 0.x" and sometimes "Luau 0.x.y"
    assert!(
        luau_version_full.starts_with("Luau 0."),
        "_VERSION global is formatted incorrectly\
        \nFound string '{luau_version_full}'"
    );
    let luau_version_noprefix = luau_version_full.strip_prefix("Luau 0.").unwrap().trim();

    // We make some guarantees about the format of the _VERSION global,
    // so make sure that the luau version also follows those rules.
    if luau_version_noprefix.is_empty() {
        panic!(
            "_VERSION global is missing version number\
            \nFound string '{luau_version_full}'"
        )
    } else if !luau_version_noprefix.chars().all(is_valid_version_char) {
        panic!(
            "_VERSION global contains invalid characters\
            \nFound string '{luau_version_full}'"
        )
    }

    luau_version_noprefix.to_string().into()
}

fn is_valid_version_char(c: char) -> bool {
    matches!(c, '0'..='9' | '.')
}
