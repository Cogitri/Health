use chrono::{Date, DateTime, FixedOffset, NaiveTime, TimeZone};
use gtk::glib::{self, Boxed, DateTime as GDateTime};

pub mod prelude {
    pub use super::*;
}

#[derive(Clone, Debug, PartialEq, Eq, Boxed)]
#[boxed_type(name = "DateTimeBoxed")]
pub struct DateTimeBoxed(pub DateTime<FixedOffset>);

#[derive(Clone, Debug, PartialEq, Eq, Boxed)]
#[boxed_type(name = "NaiveTimeBoxed")]
pub struct NaiveTimeBoxed(pub NaiveTime);

#[easy_ext::ext(DateTimeExt)]
impl<T> DateTime<T>
where
    T: TimeZone,
    T::Offset: std::fmt::Display,
{
    pub fn format_local(&self) -> String {
        GDateTime::from_iso8601(&self.to_rfc3339(), None)
            .unwrap()
            .format("%x")
            .unwrap()
            .to_string()
    }
}

#[easy_ext::ext(DateExt)]
impl<T> Date<T>
where
    T: TimeZone,
    T::Offset: std::fmt::Display,
{
    pub fn format_local(&self) -> String {
        GDateTime::from_iso8601(&self.and_hms(0, 0, 0).to_rfc3339(), None)
            .unwrap()
            .format("%x")
            .unwrap()
            .to_string()
    }
}

#[easy_ext::ext(GDateTimeExt)]
impl glib::DateTime {
    pub fn to_chrono(&self) -> chrono::DateTime<FixedOffset> {
        DateTime::parse_from_str(
            &self.format("%Y-%m-%d %H:%M:%S %z").unwrap(),
            "%Y-%m-%d %H:%M:%S %#z",
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{DateTime, Datelike, FixedOffset, Timelike};

    #[test]
    fn convert_gdatetime_chrono() {
        let gdate = glib::DateTime::now_local().unwrap();
        let chrono_date: DateTime<FixedOffset> = gdate.to_chrono();
        assert_eq!(gdate.day_of_month() as u32, chrono_date.day());
        assert_eq!(gdate.year(), chrono_date.year());
        assert_eq!(gdate.month() as u32, chrono_date.month());
        assert_eq!(gdate.hour() as u32, chrono_date.hour());
        assert_eq!(gdate.minute() as u32, chrono_date.minute());
        assert_eq!(gdate.second() as u32, chrono_date.second());
    }
}
