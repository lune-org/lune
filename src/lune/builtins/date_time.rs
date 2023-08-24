use chrono::prelude::*;
use chrono::DateTime as ChronoDateTime;
use chrono_locale::LocaleDate;
use once_cell::sync::Lazy;

// TODO: Proper error handling and stuff

/// Possible types of timestamps accepted by `DateTime`.
pub enum TimestampType {
    Seconds,
    Millis,
}

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
