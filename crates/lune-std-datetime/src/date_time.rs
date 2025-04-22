use std::cmp::Ordering;

use mlua::prelude::*;

use chrono::prelude::*;
use chrono::DateTime as ChronoDateTime;
use chrono_lc::LocaleDate;

use crate::result::{DateTimeError, DateTimeResult};
use crate::values::DateTimeValues;

const DEFAULT_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const DEFAULT_LOCALE: &str = "en";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    // NOTE: We store this as the UTC time zone since it is the most commonly
    // used and getting the generics right for TimeZone is somewhat tricky,
    // but none of the method implementations below should rely on this tz
    inner: ChronoDateTime<Utc>,
}

impl DateTime {
    /**
        Creates a new `DateTime` struct representing the current moment in time.

        See [`chrono::DateTime::now`] for additional details.
    */
    #[must_use]
    pub fn now() -> Self {
        Self { inner: Utc::now() }
    }

    /**
        Creates a new `DateTime` struct from the given `unix_timestamp`,
        which is a float of seconds passed since the UNIX epoch.

        This is somewhat unconventional, but fits our Luau interface and dynamic types quite well.
        To use this method the same way you would use a more traditional `from_unix_timestamp`
        that takes a `u64` of seconds or similar type, casting the value is sufficient:

        ```rust ignore
        DateTime::from_unix_timestamp_float(123456789u64 as f64)
        ```

        See [`chrono::DateTime::from_timestamp`] for additional details.

        # Errors

        Returns an error if the input value is out of range.
    */
    pub fn from_unix_timestamp_float(unix_timestamp: f64) -> DateTimeResult<Self> {
        let whole = unix_timestamp.trunc() as i64;
        let fract = unix_timestamp.fract();
        let nanos = (fract * 1_000_000_000f64)
            .round()
            .clamp(u32::MIN as f64, u32::MAX as f64) as u32;
        let inner = ChronoDateTime::<Utc>::from_timestamp(whole, nanos)
            .ok_or(DateTimeError::OutOfRangeUnspecified)?;
        Ok(Self { inner })
    }

    /**
        Transforms individual date & time values into a new
        `DateTime` struct, using the universal (UTC) time zone.

        See [`chrono::NaiveDate::from_ymd_opt`] and [`chrono::NaiveTime::from_hms_milli_opt`]
        for additional details and cases where this constructor may return an error.

        # Errors

        Returns an error if the date or time values are invalid.
    */
    pub fn from_universal_time(values: &DateTimeValues) -> DateTimeResult<Self> {
        let date = NaiveDate::from_ymd_opt(values.year, values.month, values.day)
            .ok_or(DateTimeError::InvalidDate)?;

        let time = NaiveTime::from_hms_milli_opt(
            values.hour,
            values.minute,
            values.second,
            values.millisecond,
        )
        .ok_or(DateTimeError::InvalidTime)?;

        let inner = Utc.from_utc_datetime(&NaiveDateTime::new(date, time));

        Ok(Self { inner })
    }

    /**
        Transforms individual date & time values into a new
        `DateTime` struct, using the current local time zone.

        See [`chrono::NaiveDate::from_ymd_opt`] and [`chrono::NaiveTime::from_hms_milli_opt`]
        for additional details and cases where this constructor may return an error.

        # Errors

        Returns an error if the date or time values are invalid or ambiguous.
    */
    pub fn from_local_time(values: &DateTimeValues) -> DateTimeResult<Self> {
        let date = NaiveDate::from_ymd_opt(values.year, values.month, values.day)
            .ok_or(DateTimeError::InvalidDate)?;

        let time = NaiveTime::from_hms_milli_opt(
            values.hour,
            values.minute,
            values.second,
            values.millisecond,
        )
        .ok_or(DateTimeError::InvalidTime)?;

        let inner = Local
            .from_local_datetime(&NaiveDateTime::new(date, time))
            .single()
            .ok_or(DateTimeError::Ambiguous)?
            .with_timezone(&Utc);

        Ok(Self { inner })
    }

    /**
        Formats the `DateTime` using the universal (UTC) time
        zone, the given format string, and the given locale.

        `format` and `locale` default to `"%Y-%m-%d %H:%M:%S"` and `"en"` respectively.

        See [`chrono_lc::DateTime::formatl`] for additional details.
    */
    #[must_use]
    pub fn format_string_local(&self, format: Option<&str>, locale: Option<&str>) -> String {
        self.inner
            .with_timezone(&Local)
            .formatl(
                format.unwrap_or(DEFAULT_FORMAT),
                locale.unwrap_or(DEFAULT_LOCALE),
            )
            .to_string()
    }

    /**
        Formats the `DateTime` using the universal (UTC) time
        zone, the given format string, and the given locale.

        `format` and `locale` default to `"%Y-%m-%d %H:%M:%S"` and `"en"` respectively.

        See [`chrono_lc::DateTime::formatl`] for additional details.
    */
    #[must_use]
    pub fn format_string_universal(&self, format: Option<&str>, locale: Option<&str>) -> String {
        self.inner
            .with_timezone(&Utc)
            .formatl(
                format.unwrap_or(DEFAULT_FORMAT),
                locale.unwrap_or(DEFAULT_LOCALE),
            )
            .to_string()
    }

    /**
        Parses a time string in the ISO 8601 format, such as
        `1996-12-19T16:39:57-08:00`, into a new `DateTime` struct.

        See [`chrono::DateTime::parse_from_rfc3339`] for additional details.

        # Errors

        Returns an error if the input string is not a valid RFC 3339 date-time.
    */
    pub fn from_iso_date(iso_date: impl AsRef<str>) -> DateTimeResult<Self> {
        let inner = ChronoDateTime::parse_from_rfc3339(iso_date.as_ref())?.with_timezone(&Utc);
        Ok(Self { inner })
    }

    /**
        Parses a time string in the RFC 2822 format, such as
        `Tue, 1 Jul 2003 10:52:37 +0200`, into a new `DateTime` struct.

        See [`chrono::DateTime::parse_from_rfc2822`] for additional details.

        # Errors

        Returns an error if the input string is not a valid RFC 2822 date-time.
    */
    pub fn from_rfc_2822_date(rfc_date: impl AsRef<str>) -> DateTimeResult<Self> {
        let inner = ChronoDateTime::parse_from_rfc2822(rfc_date.as_ref())?.with_timezone(&Utc);
        Ok(Self { inner })
    }

    /**
        Extracts individual date & time values from this
        `DateTime`, using the current local time zone.
    */
    #[must_use]
    pub fn to_local_time(self) -> DateTimeValues {
        DateTimeValues::from(self.inner.with_timezone(&Local))
    }

    /**
        Extracts individual date & time values from this
        `DateTime`, using the universal (UTC) time zone.
    */
    #[must_use]
    pub fn to_universal_time(self) -> DateTimeValues {
        DateTimeValues::from(self.inner.with_timezone(&Utc))
    }

    /**
        Formats a time string in the ISO 8601 format, such as `1996-12-19T16:39:57-08:00`.

        See [`chrono::DateTime::to_rfc3339`] for additional details.
    */
    #[must_use]
    pub fn to_iso_date(self) -> String {
        self.inner.to_rfc3339()
    }

    /**
        Formats a time string in the RFC 2822 format, such as `Tue, 1 Jul 2003 10:52:37 +0200`.

        See [`chrono::DateTime::to_rfc2822`] for additional details.
    */
    #[must_use]
    pub fn to_rfc_2822_date(self) -> String {
        self.inner.to_rfc2822()
    }
}

impl LuaUserData for DateTime {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("unixTimestamp", |_, this| Ok(this.inner.timestamp()));
        fields.add_field_method_get("unixTimestampMillis", |_, this| {
            Ok(this.inner.timestamp_millis())
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Metamethods to compare DateTime as instants in time
        methods.add_meta_method(
            LuaMetaMethod::Eq,
            |_, this: &Self, other: LuaUserDataRef<Self>| Ok(this.eq(&other)),
        );
        methods.add_meta_method(
            LuaMetaMethod::Lt,
            |_, this: &Self, other: LuaUserDataRef<Self>| {
                Ok(matches!(this.cmp(&other), Ordering::Less))
            },
        );
        methods.add_meta_method(
            LuaMetaMethod::Le,
            |_, this: &Self, other: LuaUserDataRef<Self>| {
                Ok(matches!(this.cmp(&other), Ordering::Less | Ordering::Equal))
            },
        );
        // Normal methods
        methods.add_method("toIsoDate", |_, this, ()| Ok(this.to_iso_date()));
        methods.add_method("toRfc3339", |_, this, ()| Ok(this.to_iso_date()));
        methods.add_method("toRfc2822", |_, this, ()| Ok(this.to_rfc_2822_date()));
        methods.add_method(
            "formatUniversalTime",
            |_, this, (format, locale): (Option<String>, Option<String>)| {
                Ok(this.format_string_universal(format.as_deref(), locale.as_deref()))
            },
        );
        methods.add_method(
            "formatLocalTime",
            |_, this, (format, locale): (Option<String>, Option<String>)| {
                Ok(this.format_string_local(format.as_deref(), locale.as_deref()))
            },
        );
        methods.add_method("toUniversalTime", |_, this: &Self, ()| {
            Ok(this.to_universal_time())
        });
        methods.add_method("toLocalTime", |_, this: &Self, ()| Ok(this.to_local_time()));
    }
}
