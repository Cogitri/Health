/* serialize.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use crate::{
    core::{Settings, UnitSystem},
    model::ActivityType,
};
use gtk::glib;
use serde::{de, Deserialize, Deserializer, Serializer};
use std::{convert::TryInto, str::FromStr};
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

    ActivityType::from_str(buf.as_str()).map_err(serde::de::Error::custom)
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<glib::DateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    glib::DateTime::from_iso8601(&buf, None).map_err(serde::de::Error::custom)
}

pub fn deserialize_distance<'de, D>(deserializer: D) -> Result<Option<Length>, D::Error>
where
    D: Deserializer<'de>,
{
    let val = f32::deserialize(deserializer)?;
    if Settings::instance().unit_system() == UnitSystem::Metric {
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

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<glib::TimeSpan, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = u32::deserialize(deserializer)?;

    Ok(glib::TimeSpan::from_minutes(buf.into()))
}

pub fn deserialize_mass<'de, D>(deserializer: D) -> Result<Mass, D::Error>
where
    D: Deserializer<'de>,
{
    let val = f32::deserialize(deserializer)?;
    if Settings::instance().unit_system() == UnitSystem::Metric {
        Ok(Mass::new::<kilogram>(val))
    } else {
        Ok(Mass::new::<pound>(val))
    }
}

pub fn deserialize_modified_time_millis<'de, D>(deserializer: D) -> Result<glib::DateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?.parse::<u64>().unwrap() / 1000;
    let date_time = glib::DateTime::from_unix_utc(val.try_into().unwrap());

    date_time.map_or_else(|_| Err(de::Error::custom("date would overflow")), Ok)
}

pub fn serialize_activity_type<S>(val: &ActivityType, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(val.as_ref())
}

pub fn serialize_date<S>(d: &glib::DateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&d.format_iso8601().unwrap().as_str())
}

#[allow(clippy::trivially_copy_pass_by_ref)]
#[allow(clippy::option_if_let_else)]
pub fn serialize_distance<S>(l: &Option<Length>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if Settings::instance().unit_system() == UnitSystem::Metric {
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

pub fn serialize_duration<S>(d: &glib::TimeSpan, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u32(d.as_minutes().try_into().unwrap())
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_mass<S>(mass: &Mass, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if Settings::instance().unit_system() == UnitSystem::Metric {
        s.serialize_f32(mass.get::<kilogram>())
    } else {
        s.serialize_f32(mass.get::<pound>())
    }
}
