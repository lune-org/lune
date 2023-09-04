use chrono::Month;
use mlua::prelude::*;

pub(crate) mod builder;
pub(crate) mod date_time;

use self::{
    builder::DateTimeBuilder,
    date_time::{DateTime, TimestampType, Timezone},
};
use crate::lune::util::TableBuilder;

// TODO: Proper error handling and stuff

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("now", |_, ()| Ok(DateTime::now()))?
        .with_function("fromUnixTimestamp", |lua, timestamp: LuaValue| {
            let timestamp_cloned = timestamp.clone();
            let timestamp_kind = TimestampType::from_lua(timestamp, lua)?;
            let timestamp = match timestamp_kind {
                TimestampType::Seconds => timestamp_cloned.as_i64().ok_or(LuaError::external("invalid float integer timestamp supplied"))?,
                TimestampType::Millis => {
                    let timestamp = timestamp_cloned.as_f64().ok_or(LuaError::external("invalid float timestamp with millis component supplied"))?;
                    ((((timestamp - timestamp.fract()) as u64) * 1000_u64) // converting the whole seconds part to millis
                    // the ..3 gets a &str of the first 3 chars of the digits after the decimals, ignoring
                    // additional floating point accuracy digits
                        + (timestamp.fract() * (10_u64.pow(timestamp.fract().to_string().split('.').collect::<Vec<&str>>()[1][..3].len() as u32)) as f64) as u64) as i64
                    // adding the millis to the fract as a whole number
                    // HACK: 10 ** (timestamp.fract().to_string().len() - 2) gives us the number of digits
                    // after the decimal
                }
            };

            Ok(DateTime::from_unix_timestamp(timestamp_kind, timestamp))
        })?
        .with_function("fromUniversalTime", |lua, date_time: LuaValue| {
            Ok(DateTime::from_universal_time(DateTimeBuilder::from_lua(date_time, lua).ok()))
        })?
        .with_function("fromLocalTime", |lua, date_time: LuaValue| {
            Ok(DateTime::from_local_time(DateTimeBuilder::from_lua(date_time, lua).ok()))
        })?
        .with_function("fromIsoDate", |_, iso_date: String| {
            Ok(DateTime::from_iso_date(iso_date))
        })?
        .build_readonly()
}

impl<'lua> FromLua<'lua> for TimestampType {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(_) => Ok(TimestampType::Seconds),
            LuaValue::Number(num) => Ok(if num.fract() == 0.0 {
                TimestampType::Seconds
            } else {
                TimestampType::Millis
            }),
            _ => Err(LuaError::external(
                "Invalid enum type, number or integer expected",
            )),
        }
    }
}

impl LuaUserData for DateTime {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("unixTimestamp", |_, this| Ok(this.unix_timestamp));
        fields.add_field_method_get("unixTimestampMillis", |_, this| {
            Ok(this.unix_timestamp_millis)
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("toIsoDate", |_, this, ()| {
            Ok(this
                .to_iso_date()
                .map_err(|()| LuaError::external("failed to parse DateTime object, invalid")))
        });

        methods.add_method(
            "formatTime",
            |_, this, (timezone, fmt_str, locale): (LuaValue, String, String)| {
                Ok(this
                    .format_time(Timezone::from_lua(timezone, &Lua::new())?, fmt_str, locale)
                    .map_err(|()| LuaError::external("failed to parse DateTime object, invalid")))
            },
        );

        methods.add_method("toUniversalTime", |_, this: &DateTime, ()| {
            Ok(this.to_universal_time())
        });

        methods.add_method("toLocalTime", |_, this: &DateTime, ()| {
            Ok(this.to_local_time())
        });
    }
}

impl<'lua> FromLua<'lua> for DateTime {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => panic!("found nil"),
            LuaValue::Table(t) => Ok(DateTime::from_unix_timestamp(
                TimestampType::Seconds,
                t.get("unixTimestamp")?,
            )),
            _ => panic!("invalid type"),
        }
    }
}

impl LuaUserData for DateTimeBuilder {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("year", |_, this| Ok(this.year));
        fields.add_field_method_get("month", |_, this| Ok(this.month));
        fields.add_field_method_get("day", |_, this| Ok(this.day));
        fields.add_field_method_get("hour", |_, this| Ok(this.hour));
        fields.add_field_method_get("minute", |_, this| Ok(this.minute));
        fields.add_field_method_get("second", |_, this| Ok(this.second));
        fields.add_field_method_get("millisecond", |_, this| Ok(this.millisecond));
    }
}

impl<'lua> FromLua<'lua> for DateTimeBuilder {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(t) => Ok(Self::default()
                .with_year(t.get("year")?)
                .with_month(
                    (match t.get("month")? {
                        LuaValue::String(str) => Ok(str.to_str()?.parse::<Month>().or(Err(
                            LuaError::external("could not cast month string to Month"),
                        ))?),
                        LuaValue::Nil => {
                            Err(LuaError::external("cannot find mandatory month argument"))
                        }
                        LuaValue::Number(num) => Ok(Month::try_from(num as u8).or(Err(
                            LuaError::external("could not cast month number to Month"),
                        ))?),
                        LuaValue::Integer(int) => Ok(Month::try_from(int as u8).or(Err(
                            LuaError::external("could not cast month integer to Month"),
                        ))?),
                        _ => Err(LuaError::external("unexpected month field type")),
                    })?,
                )
                .with_day(t.get("day")?)
                .with_hour(t.get("hour")?)
                .with_minute(t.get("minute")?)
                .with_second(t.get("second")?)
                .with_millisecond(t.get("millisecond")?)
                .build()),
            _ => Err(LuaError::external(
                "expected type table for DateTimeBuilder",
            )),
        }
    }
}

impl<'lua> FromLua<'lua> for Timezone {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => match str.to_str()?.to_lowercase().as_str() {
                "utc" => Ok(Timezone::Utc),
                "local" => Ok(Timezone::Local),
                &_ => Err(LuaError::external("Invalid enum member!")),
            },
            _ => Err(LuaError::external("Invalid enum type, string expected")),
        }
    }
}
