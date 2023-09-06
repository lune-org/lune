use crate::lune::builtins::datetime::date_time::Timezone;
use chrono::prelude::*;
use chrono_lc::LocaleDate;

#[derive(Copy, Clone, Debug)]
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
    pub fn to_string<T>(
        self,
        timezone: Timezone,
        format: Option<T>,
        locale: Option<T>,
    ) -> Result<String, ()>
    where
        T: ToString,
    {
        let format = match format {
            Some(fmt) => fmt.to_string(),
            None => "%Y-%m-%dT%H:%M:%SZUTC+%z".to_string(),
        };

        let locale = match locale {
            Some(locale) => locale.to_string(),
            None => "en".to_string(),
        };

        let time = Utc
            .with_ymd_and_hms(
                self.year,
                self.month,
                self.day,
                self.hour,
                self.minute,
                self.second,
            )
            .single()
            .ok_or(())?;

        // dbg!(
        //     "{}",
        //     match timezone {
        //         Timezone::Utc => time.to_rfc3339(), //.formatl((format).as_str(), locale.as_str()),
        //         Timezone::Local => time.with_timezone(&Local).to_rfc3339(), // .formatl((format).as_str(), locale.as_str()),
        //     }
        // );

        Ok(match timezone {
            Timezone::Utc => time.formatl((format).as_str(), locale.as_str()),
            Timezone::Local => time
                .with_timezone(&Local)
                .formatl((format).as_str(), locale.as_str()),
        }
        .to_string())

        // .formatl((format).as_str(), locale.as_str())
        // .to_string())

        // Ok(match timezone {
        //     Timezone::Utc => Utc
        //         .with_ymd_and_hms(
        //             self.year,
        //             self.month,
        //             self.day,
        //             self.hour,
        //             self.minute,
        //             self.second,
        //         )
        //         .single()
        //         .ok_or(())?
        //         .with_timezone(&match timezone {
        //             Timezone::Utc => Utc,
        //             Timezone::Local => Local
        //         })
        //         .formatl((format).as_str(), locale.as_str())
        //         .to_string(),
        //     Timezone::Local => Local
        //         .with_ymd_and_hms(
        //             self.year,
        //             self.month,
        //             self.day,
        //             self.hour,
        //             self.minute,
        //             self.second,
        //         )
        //         .single()
        //         .ok_or(())?
        //         .formatl((format).as_str(), locale.as_str())
        //         .to_string(),
        // })
    }

    pub fn build(self) -> Self {
        self
    }
}
