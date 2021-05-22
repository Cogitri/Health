use chrono::{Date, DateTime, FixedOffset, TimeZone};
use glib::DateTime as GDateTime;
use glib::GBoxed;

pub mod prelude {
    pub use super::*;
}

#[derive(Clone, Debug, PartialEq, Eq, GBoxed)]
#[gboxed(type_name = "DateTimeBoxed")]
pub struct DateTimeBoxed(pub DateTime<FixedOffset>);

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
