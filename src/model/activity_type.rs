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

/// All supported [ActivityType]s are listed in this enum.
#[derive(
    Debug,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    Clone,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
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
    Rollerblading,
    Running,
    Skiing,
    Soccer,
    Softball,
    Swimming,
    Tennis,
    TrackAndField,
    Volleyball,
    Walking,
}

impl Default for ActivityType {
    fn default() -> Self {
        Self::Walking
    }
}

#[cfg(test)]
mod test {
    use super::ActivityType;
    use std::str::FromStr;

    #[test]
    fn deserialize_activity_type() {
        assert_eq!(
            ActivityType::from_str("basketball"),
            Ok(ActivityType::Basketball)
        );
        assert_eq!(
            ActivityType::from_str("rollerblading"),
            Ok(ActivityType::Rollerblading),
        );
        assert_eq!(
            ActivityType::from_str("volleyball"),
            Ok(ActivityType::Volleyball),
        );
        assert_eq!(
            ActivityType::from_str("horse_riding"),
            Ok(ActivityType::HorseRiding),
        );
        assert_eq!(
            ActivityType::from_str("track_and_field"),
            Ok(ActivityType::TrackAndField),
        );
        assert!(ActivityType::from_str("unknown").is_err());
    }

    #[test]
    fn serialize_activity_type() {
        let a: &str = ActivityType::Basketball.into();
        assert_eq!(a, "basketball");
        let a: &str = ActivityType::Rollerblading.into();
        assert_eq!(a, "rollerblading");
        let a: &str = ActivityType::Volleyball.into();
        assert_eq!(a, "volleyball");
        let a: &str = ActivityType::TrackAndField.into();
        assert_eq!(a, "track_and_field");
        let a: &str = ActivityType::HorseRiding.into();
        assert_eq!(a, "horse_riding");
    }
}
