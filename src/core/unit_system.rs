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
pub enum UnitSystem {
    Imperial,
    Metric,
}

impl glib::ToValue for UnitSystem {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as glib::StaticType>::static_type()
    }
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
        assert_eq!(UnitSystem::Imperial.as_ref(), "imperial");
        assert_eq!(UnitSystem::Metric.as_ref(), "metric");
    }
}
