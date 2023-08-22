use chrono::prelude::*;
use chrono::DateTime as ChronoDateTime;

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

    pub fn from_universal_time(date_time: Option<DateTimeConstructor>) -> Self {
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

    pub fn from_local_time(date_time: Option<DateTimeConstructor>) -> Self {
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

    pub fn from_iso_date<T>(iso_date: T) -> Self
    where
        T: ToString,
    {
        let time =
            ChronoDateTime::parse_from_str(iso_date.to_string().as_str(), "%Y-%m-%dT%H:%M:%SZ")
                .expect("invalid ISO 8601 string");

        Self {
            unix_timestamp: time.timestamp(),
            unix_timestamp_millis: time.timestamp_millis(),
        }
    }

    fn to_datetime_constructor<T>(date_time: ChronoDateTime<T>) -> DateTimeConstructor
    where
        T: TimeZone,
    {
        let mut date_time_constructor = DateTimeConstructor::default();

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

    pub fn to_local_time(&self) -> DateTimeConstructor {
        Self::to_datetime_constructor(Local.timestamp_opt(self.unix_timestamp, 0).unwrap())
    }

    pub fn to_universal_time(&self) -> DateTimeConstructor {
        Self::to_datetime_constructor(Utc.timestamp_opt(self.unix_timestamp, 0).unwrap())
    }

    pub fn to_iso_date(&self) -> String {
        self.to_universal_time().to_iso_string(Timezone::UTC)
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

// fn match_month_to_num(month: Month) {
//     match month {
//         Month::January => 1,
//         Month::February => 2,
//         Month::March => 3,
//         Month::April => 4,
//         Month::May => 5,
//         Month::June => 6,
//         Month::July => 7,
//         Month::August => 8,
//         Month::September => 9,
//         Month::October => 10,
//         Month::November => 11,
//         Month::December => 12,
//     };
// }

enum Timezone {
    UTC,
    Local,
}

impl DateTimeConstructor {
    pub fn with_year(&mut self, year: i32) -> &mut Self {
        self.year = year;

        self
    }

    pub fn with_month(&mut self, month: Month) -> &mut Self {
        self.month = month as u32;

        self
    }

    pub fn with_day(&mut self, day: u32) -> &mut Self {
        self.day = day;

        self
    }

    pub fn with_hour(&mut self, hour: u32) -> &mut Self {
        self.hour = hour;

        self
    }

    pub fn with_minute(&mut self, minute: u32) -> &mut Self {
        self.minute = minute;

        self
    }

    pub fn with_second(&mut self, second: u32) -> &mut Self {
        self.second = second;

        self
    }

    pub fn with_millisecond(&mut self, millisecond: u32) -> &mut Self {
        self.millisecond = millisecond;

        self
    }

    fn to_iso_string(&self, timezone: Timezone) -> String {
        let iso_format = "%Y-%m-%dT%H:%M:%SZ";

        match timezone {
            Timezone::UTC => Utc
                .with_ymd_and_hms(
                    self.year,
                    self.month,
                    self.day,
                    self.hour,
                    self.minute,
                    self.second,
                )
                .unwrap()
                .format(iso_format)
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
                .format(iso_format)
                .to_string(),
        }
    }
}
