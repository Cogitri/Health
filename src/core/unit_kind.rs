#[derive(
    PartialEq,
    Debug,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    strum::EnumString,
    strum::IntoStaticStr,
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
