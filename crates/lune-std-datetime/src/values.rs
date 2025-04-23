use mlua::prelude::*;

use chrono::prelude::*;

use lune_utils::TableBuilder;

use super::result::{DateTimeError, DateTimeResult};

#[derive(Debug, Clone, Copy)]
pub struct DateTimeValues {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millisecond: u32,
}

impl DateTimeValues {
    /**
        Verifies that all of the date & time values are within allowed ranges:

        | Name          | Range          |
        |---------------|----------------|
        | `year`        | `1400 -> 9999` |
        | `month`       | `1 -> 12`      |
        | `day`         | `1 -> 31`      |
        | `hour`        | `0 -> 23`      |
        | `minute`      | `0 -> 59`      |
        | `second`      | `0 -> 60`      |
        | `millisecond` | `0 -> 999`     |
    */
    pub fn verify(self) -> DateTimeResult<Self> {
        verify_in_range("year", self.year, 1400, 9999)?;
        verify_in_range("month", self.month, 1, 12)?;
        verify_in_range("day", self.day, 1, 31)?;
        verify_in_range("hour", self.hour, 0, 23)?;
        verify_in_range("minute", self.minute, 0, 59)?;
        verify_in_range("second", self.second, 0, 60)?;
        verify_in_range("millisecond", self.millisecond, 0, 999)?;
        Ok(self)
    }
}

fn verify_in_range<T>(name: &'static str, value: T, min: T, max: T) -> DateTimeResult<T>
where
    T: PartialOrd + std::fmt::Display,
{
    assert!(max > min);
    if value < min || value > max {
        Err(DateTimeError::OutOfRange {
            name,
            min: min.to_string(),
            max: max.to_string(),
            value: value.to_string(),
        })
    } else {
        Ok(value)
    }
}

/*
    Conversion methods between `DateTimeValues` and plain lua tables

    Note that the `IntoLua` implementation here uses a read-only table,
    since we generally want to convert into lua when we know we have
    a fixed point in time, and we guarantee that it doesn't change
*/

impl FromLua<'_> for DateTimeValues {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if !value.is_table() {
            return Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "DateTimeValues",
                message: Some("value must be a table".to_string()),
            });
        }

        let value = value.as_table().unwrap();
        let values = Self {
            year: value.get("year")?,
            month: value.get("month")?,
            day: value.get("day")?,
            hour: value.get("hour")?,
            minute: value.get("minute")?,
            second: value.get("second")?,
            millisecond: value.get("millisecond").unwrap_or(0),
        };

        match values.verify() {
            Ok(dt) => Ok(dt),
            Err(e) => Err(LuaError::FromLuaConversionError {
                from: "table",
                to: "DateTimeValues",
                message: Some(e.to_string()),
            }),
        }
    }
}

impl IntoLua<'_> for DateTimeValues {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let tab = TableBuilder::new(lua)?
            .with_value("year", self.year)?
            .with_values(vec![
                ("month", self.month),
                ("day", self.day),
                ("hour", self.hour),
                ("minute", self.minute),
                ("second", self.second),
                ("millisecond", self.millisecond),
            ])?
            .build_readonly()?;
        Ok(LuaValue::Table(tab))
    }
}

/*
    Conversion methods between chrono's timezone-aware `DateTime` to
    and from our non-timezone-aware `DateTimeValues` values struct
*/

impl<T: TimeZone> From<DateTime<T>> for DateTimeValues {
    fn from(value: DateTime<T>) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
            hour: value.hour(),
            minute: value.minute(),
            second: value.second(),
            millisecond: value.timestamp_subsec_millis(),
        }
    }
}

impl TryFrom<DateTimeValues> for DateTime<Utc> {
    type Error = DateTimeError;
    fn try_from(value: DateTimeValues) -> Result<Self, Self::Error> {
        Utc.with_ymd_and_hms(
            value.year,
            value.month,
            value.day,
            value.hour,
            value.minute,
            value.second,
        )
        .single()
        .ok_or(DateTimeError::Ambiguous)
    }
}

impl TryFrom<DateTimeValues> for DateTime<Local> {
    type Error = DateTimeError;
    fn try_from(value: DateTimeValues) -> Result<Self, Self::Error> {
        Local
            .with_ymd_and_hms(
                value.year,
                value.month,
                value.day,
                value.hour,
                value.minute,
                value.second,
            )
            .single()
            .ok_or(DateTimeError::Ambiguous)
    }
}
