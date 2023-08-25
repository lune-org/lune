use crate::lune::builtins::LuaUserData;
use crate::lune::util::TableBuilder;
use chrono::prelude::*;
use chrono::DateTime as ChronoDateTime;
use chrono_locale::LocaleDate;
use mlua::prelude::*;
use once_cell::sync::Lazy;

// TODO: Proper error handling and stuff
// TODO: fromUniversalTime, fromLocalTime, toDateTimeBuilder, toLocalTime, toUniversalTime
// FIX: DateTime::from_iso_date is broken

//   pub fn format_time<T>(&self, timezone: Timezone, fmt_str: T, locale: T) -> String
//    where
//        T: ToString,
pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("now", |_, ()| Ok(DateTime::now()))?
        .with_function("fromUnixTimestamp", |lua, (timestamp_type, timestamp)| {
            Ok(DateTime::from_unix_timestamp(
                TimestampType::from_lua(timestamp_type, lua)?,
                timestamp,
            ))
        })?
        .with_function("fromIsoDate", |_, iso_date: LuaString| {
            Ok(DateTime::from_iso_date(iso_date.to_string_lossy()))
        })?
        .with_function("toIsoDate", |_, this| Ok(DateTime::to_iso_date(&this)))?
        .with_function(
            "formatTime",
            |_, (this, timezone, fmt_str, locale): (DateTime, LuaValue, LuaString, LuaString)| {
                Ok(this.format_time(
                    Timezone::from_lua(timezone, lua)?,
                    fmt_str.to_string_lossy(),
                    locale.to_string_lossy(),
                ))
            },
        )?
        .build_readonly()
}

/// Possible types of timestamps accepted by `DateTime`.
pub enum TimestampType {
    Seconds,
    Millis,
}

impl<'lua> FromLua<'lua> for TimestampType {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        fn num_to_enum(num: i32) -> LuaResult<TimestampType> {
            match num {
                1 => Ok(TimestampType::Seconds),
                2 => Ok(TimestampType::Millis),
                _ => Err(LuaError::external("Invalid enum member!")),
            }
        }

        match value {
            LuaValue::Integer(num) => num_to_enum(num),
            LuaValue::Number(num) => num_to_enum(num as i32),
            LuaValue::String(str) => match str.to_str()?.to_lowercase().as_str() {
                "seconds" | "sec" | "secs" => Ok(TimestampType::Seconds),
                "millis" | "milliseconds" | "milli" => Ok(TimestampType::Millis),
                &_ => Err(LuaError::external("Invalid enum member!")),
            },
            _ => Err(LuaError::external(
                "Invalid enum type, number or string expected",
            )),
        }
    }
}

#[derive(Clone)]
pub struct DateTime {
    /// The number of **seconds** since January 1st, 1970
    /// at 00:00 UTC (the Unix epoch). Range is
    /// -17,987,443,200 to 253,402,300,799, approximately
    /// years 1400–9999.
    pub unix_timestamp: i64,

    /// The number of **milliseconds* since January 1st, 1970
    /// at 00:00 UTC (the Unix epoch). Range is -17,987,443,200,000
    /// to 253,402,300,799,999, approximately years 1400–9999.
    pub unix_timestamp_millis: i64,
}

impl DateTime {
    /// Returns a `DateTime` representing the current moment in time.
    pub fn now() -> Self {
        let time = Utc::now();

        Self {
            unix_timestamp: time.timestamp(),
            unix_timestamp_millis: time.timestamp_millis(),
        }
    }

    /// Returns a new `DateTime` object from the given unix timestamp, in either seconds on
    /// milliseconds. In case of failure, defaults to the  (seconds or
    /// milliseconds) since January 1st, 1970 at 00:00 (UTC).
    pub fn from_unix_timestamp(timestamp_kind: TimestampType, unix_timestamp: i64) -> Self {
        let time_chrono = match timestamp_kind {
            TimestampType::Seconds => NaiveDateTime::from_timestamp_opt(unix_timestamp, 0),
            TimestampType::Millis => NaiveDateTime::from_timestamp_millis(unix_timestamp),
        };

        if let Some(time) = time_chrono {
            Self {
                unix_timestamp: time.timestamp(),
                unix_timestamp_millis: time.timestamp_millis(),
            }
        } else {
            Self::now()
        }
    }

    /// Returns a new `DateTime` using the given units from a UTC time. The
    /// values accepted are similar to those found in the time value table
    /// returned by `to_universal_time`.
    ///
    /// - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    /// - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    /// - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    /// - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.
    pub fn from_universal_time(date_time: Option<DateTimeBuilder>) -> Self {
        if let Some(date_time) = date_time {
            let utc_time: ChronoDateTime<Utc> = Utc.from_utc_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(date_time.year, date_time.month, date_time.day)
                    .expect("invalid date"),
                NaiveTime::from_hms_milli_opt(
                    date_time.hour,
                    date_time.minute,
                    date_time.second,
                    date_time.millisecond,
                )
                .expect("invalid time"),
            ));

            Self {
                unix_timestamp: utc_time.timestamp(),
                unix_timestamp_millis: utc_time.timestamp_millis(),
            }
        } else {
            let utc_time = Utc::now();

            Self {
                unix_timestamp: utc_time.timestamp(),
                unix_timestamp_millis: utc_time.timestamp_millis(),
            }
        }
    }

    /// Returns a new `DateTime` using the given units from a UTC time. The
    /// values accepted are similar to those found in the time value table
    /// returned by `to_local_time`.
    ///
    /// - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    /// - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    /// - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    /// - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.
    pub fn from_local_time(date_time: Option<DateTimeBuilder>) -> Self {
        if let Some(date_time) = date_time {
            let local_time: ChronoDateTime<Local> = Local
                .from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(date_time.year, date_time.month, date_time.day)
                        .expect("invalid date"),
                    NaiveTime::from_hms_milli_opt(
                        date_time.hour,
                        date_time.minute,
                        date_time.second,
                        date_time.millisecond,
                    )
                    .expect("invalid time"),
                ))
                .unwrap();

            Self {
                unix_timestamp: local_time.timestamp(),
                unix_timestamp_millis: local_time.timestamp_millis(),
            }
        } else {
            let local_time = Local::now();

            Self {
                unix_timestamp: local_time.timestamp(),
                unix_timestamp_millis: local_time.timestamp_millis(),
            }
        }
    }

    /// Returns a `DateTime` from an ISO 8601 date-time string in UTC
    /// time, such as those returned by `to_iso_date`. If the
    /// string parsing fails, the function returns `None`.
    ///
    /// An example ISO 8601 date-time string would be `2020-01-02T10:30:45Z`,
    /// which represents January 2nd 2020 at 10:30 AM, 45 seconds.
    pub fn from_iso_date<T>(iso_date: T) -> Option<Self>
    where
        T: ToString,
    {
        let time =
            ChronoDateTime::parse_from_str(iso_date.to_string().as_str(), "%Y-%m-%dT%H:%M:%SZ")
                .ok()?;

        Some(Self {
            unix_timestamp: time.timestamp(),
            unix_timestamp_millis: time.timestamp_millis(),
        })
    }

    /// Converts the value of this `DateTime` object to local time. The returned table
    /// contains the following keys: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`,
    /// `Millisecond`. For more details, see the time value table in this data type's
    /// description. The values within this table could be passed to `from_local_time`
    /// to produce the original `DateTime` object.
    pub fn to_datetime_builder<T>(date_time: ChronoDateTime<T>) -> DateTimeBuilder
    where
        T: TimeZone,
    {
        let mut date_time_constructor = DateTimeBuilder::default();

        // Any less tedious way to get Enum member based on index?
        // I know there's some crates available with derive macros for this,
        // would it be okay if used some of them?
        date_time_constructor
            .with_year(date_time.year())
            .with_month(match date_time.month() {
                1 => Month::January,
                2 => Month::February,
                3 => Month::March,
                4 => Month::April,
                5 => Month::May,
                6 => Month::June,
                7 => Month::July,
                8 => Month::August,
                9 => Month::September,
                10 => Month::October,
                11 => Month::November,
                12 => Month::December,
                _ => panic!("invalid month ordinal"),
            })
            .with_day(date_time.day())
            .with_hour(date_time.hour())
            .with_minute(date_time.minute())
            .with_second(date_time.second());

        date_time_constructor
    }

    /// Converts the value of this `DateTime` object to local time. The returned table
    /// contains the following keys: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`,
    /// `Millisecond`. For more details, see the time value table in this data type's
    /// description. The values within this table could be passed to `from_local_time`
    /// to produce the original `DateTime` object.
    pub fn to_local_time(&self) -> DateTimeBuilder {
        Self::to_datetime_builder(Local.timestamp_opt(self.unix_timestamp, 0).unwrap())
    }

    /// Converts the value of this `DateTime` object to Universal Coordinated Time (UTC).
    /// The returned table contains the following keys: `Year`, `Month`, `Day`, `Hour`,
    /// `Minute`, `Second`, `Millisecond`. For more details, see the time value table
    /// in this data type's description. The values within this table could be passed
    /// to `from_universal_time` to produce the original `DateTime` object.
    pub fn to_universal_time(&self) -> DateTimeBuilder {
        Self::to_datetime_builder(Utc.timestamp_opt(self.unix_timestamp, 0).unwrap())
    }

    /// Formats a date as a ISO 8601 date-time string. The value returned by this
    /// function could be passed to `from_local_time` to produce the original `DateTime`
    /// object.
    pub fn to_iso_date(&self) -> String {
        self.to_universal_time()
            .to_string::<&str>(Timezone::Utc, None, None)
    }

    // There seems to be only one localization crate for chrono,
    // which has been committed to last 5 years ago. Thus, this crate doesn't
    // work with the version of chrono we're using. I've forked the crate
    // and have made it compatible with the latest version of chrono. ~ DevComp

    // TODO: Implement more locales for chrono-locale.

    /// Generates a string from the `DateTime` value interpreted as **local time**
    /// and a format string. The format string should contain tokens, which will
    /// replace to certain date/time values described by the `DateTime` object.
    /// For more details, see the [accepted formatter tokens](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).
    pub fn format_time<T>(&self, timezone: Timezone, fmt_str: T, locale: T) -> String
    where
        T: ToString,
    {
        self.to_universal_time().to_string(
            timezone,
            Some(fmt_str.to_string()),
            Some(locale.to_string()),
        )
    }
}

impl LuaUserData for DateTime {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("unixTimestamp", |_, this| Ok(this.unix_timestamp));
        fields.add_field_method_get("unixTimestampMillis", |_, this| {
            Ok(this.unix_timestamp_millis)
        });

        fields.add_field_method_set("unixTimestamp", |_, this, val| {
            this.unix_timestamp = val;

            Ok(())
        });

        fields.add_field_method_set("unixTimestampMillis", |_, this, val| {
            this.unix_timestamp_millis = val;

            Ok(())
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("now", |_, _this, ()| Ok(DateTime::now()));
        methods.add_method("toIsoDate", |_, this, ()| Ok(this.to_iso_date()));
        methods.add_method(
            "formatTime",
            |_, this, (timezone, fmt_str, locale): (LuaValue, LuaString, LuaString)| {
                Ok(this.format_time(
                    Timezone::from_lua(timezone, &Lua::new())?,
                    fmt_str.to_string_lossy(),
                    locale.to_string_lossy(),
                ))
            },
        );
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

/// General timezone types accepted by `DateTime` methods.
pub enum Timezone {
    Utc,
    Local,
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
        self.month = month as u32;

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
    fn to_string<T>(&self, timezone: Timezone, format: Option<T>, locale: Option<T>) -> String
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
}

impl LuaUserData for DateTimeBuilder {}
