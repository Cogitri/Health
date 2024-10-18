use gtk::glib;
use gtk::prelude::*;

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    strum::EnumString,
    strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum UnitKind {
    /// Centimeters or Inch
    LikeCentimeters,
    /// Meters or Feet
    LikeMeters,
    /// Kilometers or Miles
    LikeKilometers,
    /// Kilograms or Pounds
    LikeKilogram,
}

impl ToValue for UnitKind {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as StaticType>::static_type()
    }
}
