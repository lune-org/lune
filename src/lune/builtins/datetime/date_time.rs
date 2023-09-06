use crate::lune::builtins::datetime::builder::DateTimeBuilder;
use chrono::prelude::*;
use chrono::DateTime as ChronoDateTime;

/// Possible types of timestamps accepted by `DateTime`.
pub enum TimestampType {
    Seconds,
    Millis,
}

/// General timezone types accepted by `DateTime` methods.
#[derive(Eq, PartialEq)]
pub enum Timezone {
    Utc,
    Local,
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
    pub fn from_universal_time(date_time: Option<DateTimeBuilder>) -> Result<Self, ()> {
        Ok(match date_time {
            Some(date_time) => {
                let utc_time: ChronoDateTime<Utc> = Utc.from_utc_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(date_time.year, date_time.month, date_time.day)
                        .ok_or(())?,
                    NaiveTime::from_hms_milli_opt(
                        date_time.hour,
                        date_time.minute,
                        date_time.second,
                        date_time.millisecond,
                    )
                    .ok_or(())?,
                ));

                Self {
                    unix_timestamp: utc_time.timestamp(),
                    unix_timestamp_millis: utc_time.timestamp_millis(),
                }
            }

            None => Self::now(),
        })
    }

    /// Returns a new `DateTime` using the given units from a local time. The
    /// values accepted are similar to those found in the time value table
    /// returned by `to_local_time`.
    ///
    /// - Date units (year, month, day) that produce an invalid date will raise an error. For example, January 32nd or February 29th on a non-leap year.
    /// - Time units (hour, minute, second, millisecond) that are outside their normal range are valid. For example, 90 minutes will cause the hour to roll over by 1; -10 seconds will cause the minute value to roll back by 1.
    /// - Non-integer values are rounded down. For example, providing 2.5 hours will be equivalent to providing 2 hours, not 2 hours 30 minutes.
    /// - Omitted values are assumed to be their lowest value in their normal range, except for year which defaults to 1970.
    pub fn from_local_time(date_time: Option<DateTimeBuilder>) -> Result<Self, ()> {
        Ok(match date_time {
            Some(date_time) => {
                let local_time: ChronoDateTime<Local> = Local
                    .from_local_datetime(&NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(date_time.year, date_time.month, date_time.day)
                            .ok_or(())?,
                        NaiveTime::from_hms_milli_opt(
                            date_time.hour,
                            date_time.minute,
                            date_time.second,
                            date_time.millisecond,
                        )
                        .ok_or(())?,
                    ))
                    .single()
                    .ok_or(())?;

                Self {
                    unix_timestamp: local_time.timestamp(),
                    unix_timestamp_millis: local_time.timestamp_millis(),
                }
            }

            None => {
                let local_time = Local::now();

                Self {
                    unix_timestamp: local_time.timestamp(),
                    unix_timestamp_millis: local_time.timestamp_millis(),
                }
            }
        })
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
        let time = ChronoDateTime::parse_from_str(
            format!("{}{}", iso_date.to_string(), "UTC+0000").as_str(),
            "%Y-%m-%dT%H:%M:%SZUTC%z",
        )
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
    pub fn to_datetime_builder<T>(date_time: ChronoDateTime<T>) -> Result<DateTimeBuilder, ()>
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
                1 => Ok(Month::January),
                2 => Ok(Month::February),
                3 => Ok(Month::March),
                4 => Ok(Month::April),
                5 => Ok(Month::May),
                6 => Ok(Month::June),
                7 => Ok(Month::July),
                8 => Ok(Month::August),
                9 => Ok(Month::September),
                10 => Ok(Month::October),
                11 => Ok(Month::November),
                12 => Ok(Month::December),
                _ => Err(()),
            }?)
            .with_day(date_time.day())
            .with_hour(date_time.hour())
            .with_minute(date_time.minute())
            .with_second(date_time.second());

        Ok(date_time_constructor)
    }

    /// Converts the value of this `DateTime` object to local time. The returned table
    /// contains the following keys: `Year`, `Month`, `Day`, `Hour`, `Minute`, `Second`,
    /// `Millisecond`. For more details, see the time value table in this data type's
    /// description. The values within this table could be passed to `from_local_time`
    /// to produce the original `DateTime` object.
    pub fn to_local_time(&self) -> Result<DateTimeBuilder, ()> {
        Self::to_datetime_builder(
            Local
                .timestamp_opt(self.unix_timestamp, 0)
                .single()
                .ok_or(())?,
        )
    }

    /// Converts the value of this `DateTime` object to Universal Coordinated Time (UTC).
    /// The returned table contains the following keys: `Year`, `Month`, `Day`, `Hour`,
    /// `Minute`, `Second`, `Millisecond`. For more details, see the time value table
    /// in this data type's description. The values within this table could be passed
    /// to `from_universal_time` to produce the original `DateTime` object.
    pub fn to_universal_time(&self) -> Result<DateTimeBuilder, ()> {
        Self::to_datetime_builder(
            Utc.timestamp_opt(self.unix_timestamp, 0)
                .single()
                .ok_or(())?,
        )

        // dbg!("{:#?}", m?);

        // m
    }

    /// Formats a date as a ISO 8601 date-time string, returns None if the DateTime object is invalid.
    /// The value returned by this function could be passed to `from_local_time` to produce the
    /// original `DateTime` object.
    pub fn to_iso_date(&self) -> Result<String, ()> {
        self.to_universal_time()?
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
    pub fn format_time<T>(&self, timezone: Timezone, fmt_str: T, locale: T) -> Result<String, ()>
    where
        T: ToString,
    {
        self.to_universal_time()?.to_string(
            timezone,
            Some(fmt_str.to_string()),
            Some(locale.to_string()),
        )
    }
}
