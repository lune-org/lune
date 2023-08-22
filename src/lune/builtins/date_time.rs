use anyhow::Result;
use chrono::prelude::*;

// TODO: Proper error handling and stuff

pub enum TimestampType {
    Seconds,
    Millis,
}

pub struct DateTime {
    pub unix_timestamp: i64,
    pub unix_timestamp_millis: i64,
}

impl DateTime {
    /// Returns a DateTime representing the current moment in time
    pub fn now() -> Self {
        let time = Utc::now();

        Self {
            unix_timestamp: time.timestamp(),
            unix_timestamp_millis: time.timestamp_millis(),
        }
    }

    /// Returns a new DateTime object from the given unix timestamp, in either seconds on
    /// milliseconds. In case of failure, defaults to the  (seconds or
    /// milliseconds) since January 1st, 1970 at 00:00 (UTC)
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

    pub fn from_local_time(date_time: Option<DateTimeConstructor>) -> Self {
        if let Some(date_time) = date_time {
            let local_time: DateTime<Local> = Local
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

    pub fn from_iso_date<T>(iso_date: T) -> Self
    where
        T: ToString,
    {
        let time = DateTime::parse_from_str(iso_date.to_string().as_str(), "%Y-%m-%dT%H:%M:%SZ")
            .expect("invalid ISO 8601 string");

        Self {
            unix_timestamp: time.timestamp(),
            unix_timestamp_millis: time.timestamp_millis(),
        }
    }
}

pub struct DateTimeConstructor {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
}

impl Default for DateTimeConstructor {
    /// Constructs the default state for DateTimeConstructor, which is the Unix Epoch.
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

pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl DateTimeConstructor {
    pub fn with_year(&mut self, year: i32) -> &Self {
        self.year = year;

        self
    }

    pub fn with_month(&mut self, month: Month) -> &Self {
        let month = match month {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        };

        self.month = month;

        self
    }

    pub fn with_day(&mut self, day: u32) -> &Self {
        self.day = day;

        self
    }

    pub fn with_hour(&mut self, hour: u32) -> &Self {
        self.hour = hour;

        self
    }

    pub fn with_minute(&mut self, minute: u32) -> &Self {
        self.minute = minute;

        self
    }

    pub fn with_second(&mut self, second: u32) -> &Self {
        self.second = second;

        self
    }

    pub fn with_millisecond(&mut self, millisecond: u32) -> &Self {
        self.millisecond = millisecond;

        self
    }
}
