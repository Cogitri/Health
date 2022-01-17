use gtk::glib;

#[derive(
    PartialEq,
    Debug,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    strum::EnumString,
    strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum NotificationFrequency {
    Hourly,
    Every4Hrs,
    Fixed,
}

impl Default for NotificationFrequency {
    fn default() -> Self {
        Self::Every4Hrs
    }
}

impl glib::ToValue for NotificationFrequency {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as glib::StaticType>::static_type()
    }
}
