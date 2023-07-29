use anyhow::{bail, Result};
use gtk::glib::{self, Boxed};
use std::fmt;

pub mod prelude {
    pub use super::*;
}

#[derive(Clone, Debug, PartialEq, Eq, Boxed)]
#[boxed_type(name = "TimeBoxed")]
pub struct TimeBoxed(pub Time);

#[easy_ext::ext(GDateTimeExt)]
impl glib::DateTime {
    #[must_use]
    /// Retrieve a local [glib::DateTime] of the current timepoint
    pub fn local() -> Self {
        // Safe to unwrap, this may only fail if the year is after 9999 according to GLib docs
        Self::now_local().unwrap()
    }

    #[must_use]
    pub fn utc() -> Self {
        Self::now_utc().unwrap()
    }

    #[must_use]
    pub fn today() -> Self {
        let date = Self::local();
        date.reset_hms()
    }

    #[must_use]
    pub fn reset_hms(&self) -> Self {
        Self::from_local(self.year(), self.month(), self.day_of_month(), 0, 0, 0.0).unwrap()
    }

    #[must_use]
    pub fn subtract(&self, timespan: glib::TimeSpan) -> Self {
        self.add(timespan.reverse()).unwrap()
    }

    #[must_use]
    pub fn format_local(&self) -> String {
        self.format("%x").unwrap().to_string()
    }

    #[must_use]
    pub fn nanoseconds(&self) -> i64 {
        i64::from(self.microsecond()) * 1000
    }

    #[must_use]
    pub fn days_of_month(year: i32, month: i32) -> i64 {
        let following_month = Self::from_local(
            match month {
                12 => year + 1,
                _ => year,
            },
            match month {
                12 => 1,
                _ => month + 1,
            },
            1,
            0,
            0,
            0.0,
        )
        .unwrap();
        let date = Self::from_local(year, month, 1, 0, 0, 0.0).unwrap();
        -date.difference(&following_month).as_days()
    }

    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        self.year() == other.year()
            && self.day_of_year() == other.day_of_year()
            && self.hour() == other.hour()
            && self.minute() == other.minute()
            && self.second() == other.second()
    }

    #[must_use]
    pub fn equals_date(&self, other: &Self) -> bool {
        self.year() == other.year() && self.day_of_year() == other.day_of_year()
    }

    #[must_use]
    pub fn date(&self) -> Date {
        Date::new(
            self.year().try_into().unwrap(),
            self.month().try_into().unwrap(),
            self.day_of_month().try_into().unwrap(),
        )
        .unwrap()
    }
}

#[easy_ext::ext(GTimeSpanExt)]
impl glib::TimeSpan {
    #[must_use]
    pub fn reverse(&self) -> Self {
        Self::from_microseconds(-self.as_microseconds())
    }

    #[must_use]
    pub fn as_years(&self) -> i64 {
        self.as_days() / 365
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Time {
    hour: u8,
    minutes: u8,
    seconds: u8,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.hour, self.minutes, self.seconds)
    }
}

impl Time {
    pub fn new(hour: u8, minutes: u8, seconds: u8) -> Result<Self> {
        if hour > 24 || minutes > 60 || seconds > 60 {
            bail!("Invalid time range!");
        }

        Ok(Self {
            hour,
            minutes,
            seconds,
        })
    }

    pub fn parse(string: &str) -> Result<Self> {
        let split: Vec<&str> = string.split(':').collect();
        if split.len() < 3 {
            bail!("Invalid string!");
        }

        Self::new(split[0].parse()?, split[1].parse()?, split[2].parse()?)
    }

    #[must_use]
    pub fn hour(&self) -> u8 {
        self.hour
    }

    #[must_use]
    pub fn minutes(&self) -> u8 {
        self.minutes
    }

    #[must_use]
    pub fn seconds(&self) -> u8 {
        self.seconds
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self> {
        if month > 12 || day > 31 {
            bail!("Invalid date!");
        }

        glib::DateTime::from_local(year.into(), month.into(), day.into(), 0, 0, 0.0)?;

        Ok(Self { year, month, day })
    }

    pub fn parse(string: &str) -> Result<Self> {
        let split: Vec<&str> = string.split('-').collect();
        if split.len() < 3 {
            anyhow::bail!("Invalid string!");
        }
        Self::new(split[0].parse()?, split[1].parse()?, split[2].parse()?)
    }

    #[must_use]
    pub fn and_time_local(&self, time: Time) -> glib::DateTime {
        glib::DateTime::from_local(
            self.year().into(),
            self.month().into(),
            self.day().into(),
            time.hour().into(),
            time.minutes().into(),
            time.seconds().into(),
        )
        .unwrap()
    }

    #[must_use]
    pub fn and_time_utc(&self, time: Time) -> glib::DateTime {
        glib::DateTime::from_utc(
            self.year().into(),
            self.month().into(),
            self.day().into(),
            time.hour().into(),
            time.minutes().into(),
            time.seconds().into(),
        )
        .unwrap()
    }

    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> u8 {
        self.month
    }

    #[must_use]
    pub fn day(&self) -> u8 {
        self.day
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_local() {
        let date = glib::DateTime::from_unix_utc(1_000_000_000).unwrap();

        if gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "en_US.UTF-8").is_some() {
            assert_eq!(date.format_local(), "09/09/2001");
        }
        if gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "de_DE.UTF-8").is_some() {
            assert_eq!(date.format_local(), "09.09.2001");
        }
    }
}
