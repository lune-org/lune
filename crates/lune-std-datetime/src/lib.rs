#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

mod date_time;
mod result;
mod values;

pub use self::date_time::DateTime;

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `datetime` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `datetime` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("fromIsoDate", |_, date: String| {
            Ok(DateTime::from_rfc_3339(date)?) // FUTURE: Remove this rfc3339 alias method
        })?
        .with_function("fromRfc3339", |_, date: String| {
            Ok(DateTime::from_rfc_3339(date)?)
        })?
        .with_function("fromRfc2822", |_, date: String| {
            Ok(DateTime::from_rfc_2822(date)?)
        })?
        .with_function("fromLocalTime", |_, values| {
            Ok(DateTime::from_local_time(&values)?)
        })?
        .with_function("fromUniversalTime", |_, values| {
            Ok(DateTime::from_universal_time(&values)?)
        })?
        .with_function("fromUnixTimestamp", |_, timestamp| {
            Ok(DateTime::from_unix_timestamp_float(timestamp)?)
        })?
        .with_function("now", |_, ()| Ok(DateTime::now()))?
        .build_readonly()
}
