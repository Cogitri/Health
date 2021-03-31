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
pub enum Unitsystem {
    Imperial,
    Metric,
}

#[cfg(test)]
mod test {
    use super::Unitsystem;
    use std::str::FromStr;

    #[test]
    fn deserialize_unitsystem() {
        assert_eq!(Unitsystem::from_str("imperial"), Ok(Unitsystem::Imperial));
        assert_eq!(Unitsystem::from_str("metric"), Ok(Unitsystem::Metric));

        assert!(Unitsystem::from_str("unknown").is_err());
    }

    #[test]
    fn serialize_unitsystem() {
        let a: &str = Unitsystem::Imperial.into();
        assert_eq!(a, "imperial");
        let b: &str = Unitsystem::Metric.into();
        assert_eq!(b, "metric");
    }
}
