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

impl glib::ToValue for UnitKind {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as glib::StaticType>::static_type()
    }
}
