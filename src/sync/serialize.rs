use crate::{
    core::{settings::Unitsystem, HealthSettings},
    model::ActivityType,
};
use chrono::{DateTime, Duration, FixedOffset, TimeZone};
use serde::{Deserialize, Deserializer, Serializer};
use std::convert::{TryFrom, TryInto};
use uom::si::{
    f32::{Length, Mass},
    length::{meter, yard},
    mass::{kilogram, pound},
};

pub fn deserialize_activity_type<'de, D>(deserializer: D) -> Result<ActivityType, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    ActivityType::try_from(buf.as_str()).map_err(serde::de::Error::custom)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    DateTime::parse_from_rfc3339(&buf).map_err(serde::de::Error::custom)
}

pub fn deserialize_distance<'de, D>(deserializer: D) -> Result<Option<Length>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = f32::deserialize(deserializer)?;
    if HealthSettings::new().get_unitsystem() == Unitsystem::Metric {
        if val == 0.0 {
            Ok(None)
        } else {
            Ok(Some(Length::new::<meter>(val)))
        }
    } else if val == 0.0 {
        Ok(None)
    } else {
        Ok(Some(Length::new::<yard>(val)))
    }
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = u32::deserialize(deserializer)?;

    Ok(Duration::minutes(buf as i64))
}

pub fn deserialize_mass<'de, D>(deserializer: D) -> Result<Mass, D::Error>
where
    D: Deserializer<'de>,
{
    let val = f32::deserialize(deserializer)?;
    if HealthSettings::new().get_unitsystem() == Unitsystem::Metric {
        Ok(Mass::new::<kilogram>(val))
    } else {
        Ok(Mass::new::<pound>(val))
    }
}

pub fn deserialize_modified_time_millis<'de, D>(
    deserializer: D,
) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?.parse::<u64>().unwrap() / 1000;
    let date_time = FixedOffset::east(0)
        .ymd(1970, 1, 1)
        .and_hms(0, 0, 0)
        .checked_add_signed(Duration::milliseconds(val.try_into().unwrap()))
        .unwrap();

    Ok(date_time)
}

pub fn serialize_activity_type<S>(val: &ActivityType, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(val.clone().into())
}

pub fn serialize_date<S>(d: &DateTime<FixedOffset>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&d.to_rfc3339())
}

pub fn serialize_distance<S>(l: &Option<Length>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if HealthSettings::new().get_unitsystem() == Unitsystem::Metric {
        if let Some(length) = l {
            s.serialize_f32(length.get::<meter>())
        } else {
            s.serialize_f32(0.0)
        }
    } else if let Some(length) = l {
        s.serialize_f32(length.get::<yard>())
    } else {
        s.serialize_f32(0.0)
    }
}

pub fn serialize_duration<S>(d: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u32(d.num_minutes() as u32)
}

pub fn serialize_mass<S>(mass: &Mass, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if HealthSettings::new().get_unitsystem() == Unitsystem::Metric {
        s.serialize_f32(mass.get::<kilogram>())
    } else {
        s.serialize_f32(mass.get::<pound>())
    }
}
