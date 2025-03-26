#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

mod date_time;
mod result;
mod values;

pub use self::date_time::DateTime;

/**
    Creates the `datetime` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("fromIsoDate", |_, iso_date: String| {
            Ok(DateTime::from_iso_date(iso_date)?)
        })?
        .with_function("fromRfcDate", |_, rfc_date: String| {
            Ok(DateTime::from_rfc_date(rfc_date)?)
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
