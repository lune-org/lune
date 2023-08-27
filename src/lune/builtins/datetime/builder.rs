use crate::lune::builtins::datetime::date_time::Timezone;
use chrono::prelude::*;
use chrono_locale::LocaleDate;
use mlua::prelude::*;
use once_cell::sync::Lazy;

#[derive(Copy, Clone)]
pub struct DateTimeBuilder {
    /// The year. In the range 1400 - 9999.
    pub year: i32,
    /// The month. In the range 1 - 12.
    pub month: u32,
    /// The day. In the range 1 - 31.
    pub day: u32,
    /// The hour. In the range 0 - 23.
    pub hour: u32,
    /// The minute. In the range 0 - 59.
    pub minute: u32,
    /// The second. In the range usually 0 - 59, but sometimes 0 - 60 to accommodate leap seconds in certain systems.
    pub second: u32,
    /// The milliseconds. In the range 0 - 999.
    pub millisecond: u32,
}

impl Default for DateTimeBuilder {
    /// Constructs the default state for DateTimeBuilder, which is the Unix Epoch.
    fn default() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }
    }
}

impl<'lua> FromLua<'lua> for Timezone {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        fn num_to_enum(num: i32) -> LuaResult<Timezone> {
            match num {
                1 => Ok(Timezone::Utc),
                2 => Ok(Timezone::Local),
                _ => Err(LuaError::external("Invalid enum member!")),
            }
        }

        match value {
            LuaValue::Integer(num) => num_to_enum(num),
            LuaValue::Number(num) => num_to_enum(num as i32),
            LuaValue::String(str) => match str.to_str()?.to_lowercase().as_str() {
                "utc" => Ok(Timezone::Utc),
                "local" => Ok(Timezone::Local),
                &_ => Err(LuaError::external("Invalid enum member!")),
            },
            _ => Err(LuaError::external(
                "Invalid enum type, number or string expected",
            )),
        }
    }
}

impl DateTimeBuilder {
    /// Builder method to set the `Year`.
    pub fn with_year(&mut self, year: i32) -> &mut Self {
        self.year = year;

        self
    }

    /// Builder method to set the `Month`.
    pub fn with_month(&mut self, month: Month) -> &mut Self {
        // THe Month enum casts to u32 starting at zero, so we add one to it
        self.month = month as u32 + 1;

        self
    }

    /// Builder method to set the `Month`.
    pub fn with_day(&mut self, day: u32) -> &mut Self {
        self.day = day;

        self
    }

    /// Builder method to set the `Hour`.
    pub fn with_hour(&mut self, hour: u32) -> &mut Self {
        self.hour = hour;

        self
    }

    /// Builder method to set the `Minute`.
    pub fn with_minute(&mut self, minute: u32) -> &mut Self {
        self.minute = minute;

        self
    }

    /// Builder method to set the `Second`.
    pub fn with_second(&mut self, second: u32) -> &mut Self {
        self.second = second;

        self
    }

    /// Builder method to set the `Millisecond`.
    pub fn with_millisecond(&mut self, millisecond: u32) -> &mut Self {
        self.millisecond = millisecond;

        self
    }

    /// Converts the `DateTimeBuilder` to a string with a specified format and locale.
    pub fn to_string<T>(self, timezone: Timezone, format: Option<T>, locale: Option<T>) -> String
    where
        T: ToString,
    {
        let format_lazy: Lazy<String, _> = Lazy::new(|| {
            if let Some(fmt) = format {
                fmt.to_string()
            } else {
                "%Y-%m-%dT%H:%M:%SZ".to_string()
            }
        });

        let locale_lazy: Lazy<String, _> = Lazy::new(|| {
            if let Some(locale) = locale {
                locale.to_string()
            } else {
                "en".to_string()
            }
        });

        match timezone {
            Timezone::Utc => Utc
                .with_ymd_and_hms(
                    self.year,
                    self.month,
                    self.day,
                    self.hour,
                    self.minute,
                    self.second,
                )
                .unwrap()
                .formatl((*format_lazy).as_str(), (*locale_lazy).as_str())
                .to_string(),
            Timezone::Local => Local
                .with_ymd_and_hms(
                    self.year,
                    self.month,
                    self.day,
                    self.hour,
                    self.minute,
                    self.second,
                )
                .unwrap()
                .formatl((*format_lazy).as_str(), (*locale_lazy).as_str())
                .to_string(),
        }
    }

    fn build(self) -> Self {
        self
    }
}

impl LuaUserData for DateTimeBuilder {}

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
                // TODO: millisecond support
                .build()),
            _ => Err(LuaError::external(
                "expected type table for DateTimeBuilder",
            )),
        }
    }
}
