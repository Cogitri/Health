use anyhow::{bail, Result};
use gtk::glib::{self, Boxed};

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
        date.difference(&following_month).as_days() * -1
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
}

#[easy_ext::ext(GTimeSpanExt)]
impl glib::TimeSpan {
    pub fn reverse(&self) -> Self {
        Self::from_microseconds(self.as_microseconds() * -1)
    }

    pub fn as_years(&self) -> i64 {
        self.as_days() / 365
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Time {
    hour: u8,
    minutes: u8,
    seconds: u8,
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
        let split: Vec<&str> = string.split(":").collect();
        if split.len() < 3 {
            bail!("Invalid string!");
        }

        Ok(Self::new(
            split[0].parse()?,
            split[1].parse()?,
            split[2].parse()?,
        )?)
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.hour, self.minutes, self.seconds)
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minutes(&self) -> u8 {
        self.minutes
    }

    pub fn seconds(&self) -> u8 {
        self.seconds
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self> {
        if month > 12 || day > 31 {
            bail!("Invalid date!");
        }

        glib::DateTime::from_local(year.into(), month.into(), day.into(), 0, 0, 0.0)?;

        Ok(Self { year, month, day })
    }

    pub fn parse(string: &str) -> anyhow::Result<Self> {
        let split: Vec<&str> = string.split("-").collect();
        if split.len() < 3 {
            anyhow::bail!("Invalid string!");
        }
        Ok(Self::new(
            split[0].parse()?,
            split[1].parse()?,
            split[2].parse()?,
        )?)
    }

    pub fn to_string(&self) -> String {
        format!("{}-{}-{}", self.year, self.month, self.day)
    }

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

    pub fn year(&self) -> u16 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_local() {
        gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "en_US.UTF-8");
        let date = glib::DateTime::from_unix_utc(1_000_000_000).unwrap();
        assert_eq!(date.format_local(), "09/09/2001");
        gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "de_DE.UTF-8");
        assert_eq!(date.format_local(), "09.09.2001");
    }
}
