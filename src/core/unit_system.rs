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
pub enum UnitSystem {
    Imperial,
    Metric,
}

#[cfg(test)]
mod test {
    use super::UnitSystem;
    use std::str::FromStr;

    #[test]
    fn deserialize_unit_system() {
        assert_eq!(UnitSystem::from_str("imperial"), Ok(UnitSystem::Imperial));
        assert_eq!(UnitSystem::from_str("metric"), Ok(UnitSystem::Metric));

        assert!(UnitSystem::from_str("unknown").is_err());
    }

    #[test]
    fn serialize_unit_system() {
        let a: &str = UnitSystem::Imperial.into();
        assert_eq!(a, "imperial");
        let b: &str = UnitSystem::Metric.into();
        assert_eq!(b, "metric");
    }
}
