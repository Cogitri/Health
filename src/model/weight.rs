/* weight.rs
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

use crate::sync::serialize;
use chrono::{DateTime, FixedOffset};
use uom::si::f32::Mass;

/// A [Weight] is a single weight measurement the user did on a certain date.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
