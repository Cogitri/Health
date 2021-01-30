use crate::sync::serialize;
use chrono::{DateTime, FixedOffset};
use uom::si::f32::Mass;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Weight {
    #[serde(serialize_with = "serialize::serialize_date")]
    #[serde(deserialize_with = "serialize::deserialize_date")]
    pub date: DateTime<FixedOffset>,
    #[serde(serialize_with = "serialize::serialize_mass")]
    #[serde(deserialize_with = "serialize::deserialize_mass")]
    pub weight: Mass,
}

impl Weight {
    pub fn new(date: DateTime<FixedOffset>, weight: Mass) -> Self {
        Self { date, weight }
    }
}
