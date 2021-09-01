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
    LengthSmall,
    /// Meters or Feet
    LengthBig,
    /// Kilometers or Miles
    LengthVeryBig,
    /// Kilograms or Pounds
    WeightBig,
}
