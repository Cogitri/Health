/* activity_type.rs
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

use crate::core::i18n_f;
use std::convert::TryFrom;

/// All supported `ActivityType`s are listed in this enum.
#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive, Clone, PartialEq)]
pub enum ActivityType {
    Basketball,
    Bicycling,
    Boxing,
    Dancing,
    Football,
    Golf,
    Hiking,
    Hockey,
    HorseRiding,
    OtherSports,
    RollerBlading,
    Running,
    Skiing,
    Soccer,
    Softball,
    Swimming,
    Tennis,
    TrackAndField,
    VolleyBall,
    Walking,
}

impl TryFrom<&str> for ActivityType {
    type Error = String;

    /// Try to convert from an `ActivityType` ID to a `ActivityType`
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "basketball" => Ok(ActivityType::Basketball),
            "bicycling" => Ok(ActivityType::Bicycling),
            "boxing" => Ok(ActivityType::Boxing),
            "dancing" => Ok(ActivityType::Dancing),
            "football" => Ok(ActivityType::Football),
            "golf" => Ok(ActivityType::Golf),
            "hiking" => Ok(ActivityType::Hiking),
            "hockey" => Ok(ActivityType::Hockey),
            "horse_riding" => Ok(ActivityType::HorseRiding),
            "other_sports" => Ok(ActivityType::OtherSports),
            "rollerblading" => Ok(ActivityType::RollerBlading),
            "running" => Ok(ActivityType::Running),
            "skiing" => Ok(ActivityType::Skiing),
            "soccer" => Ok(ActivityType::Soccer),
            "softball" => Ok(ActivityType::Softball),
            "swimming" => Ok(ActivityType::Swimming),
            "tennis" => Ok(ActivityType::Tennis),
            "track_and_field" => Ok(ActivityType::TrackAndField),
            "volleyball" => Ok(ActivityType::VolleyBall),
            "walking" => Ok(ActivityType::Walking),
            _ => Err(i18n_f("Unknown ActivityType {}", &[value])),
        }
    }
}

impl Into<&'static str> for ActivityType {
    /// Convert from an `ActivityType` to an ID.
    fn into(self) -> &'static str {
        match self {
            ActivityType::Basketball => "basketball",
            ActivityType::Bicycling => "bicycling",
            ActivityType::Boxing => "boxing",
            ActivityType::Dancing => "dancing",
            ActivityType::Football => "football",
            ActivityType::Golf => "golf",
            ActivityType::Hiking => "hiking",
            ActivityType::Hockey => "hockey",
            ActivityType::HorseRiding => "horse_riding",
            ActivityType::OtherSports => "other_sports",
            ActivityType::RollerBlading => "rollerblading",
            ActivityType::Running => "running",
            ActivityType::Skiing => "skiing",
            ActivityType::Soccer => "soccer",
            ActivityType::Softball => "softball",
            ActivityType::Swimming => "swimming",
            ActivityType::Tennis => "tennis",
            ActivityType::TrackAndField => "track_and_field",
            ActivityType::VolleyBall => "volleyball",
            ActivityType::Walking => "walking",
        }
    }
}
