/* activity_info.rs
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
    core::i18n,
    model::{ActivityDataPoints, ActivityType},
};
use gtk::gdk::RGBA;
use gtk::glib::{self, Boxed};
use std::{convert::TryFrom, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Boxed)]
#[boxed_type(name = "ActivityInfo")]
pub struct ActivityInfoBoxed(pub ActivityInfo);

/// A struct containing informations about a certain activity type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityInfo {
    pub activity_type: ActivityType,
    pub available_data_points: ActivityDataPoints,
    pub average_calories_burned_per_minute: u32,
    /// An ID that can be used for saving things to the DB.
    pub id: &'static str,
    /// The localised name of the `ActivityType`.
    pub name: String,
    pub color: RGBA,
}

impl Default for ActivityInfo {
    fn default() -> Self {
        Self::from(ActivityType::default())
    }
}

fn rgb(r: u8, g: u8, b: u8) -> RGBA {
    gtk::gdk::RGBA::builder()
        .red(f32::from(r) / 255.0)
        .blue(f32::from(g) / 255.0)
        .green(f32::from(b) / 255.0)
        .alpha(0.9)
        .build()
}

impl From<ActivityType> for ActivityInfo {
    /// Converts an [ActivityType] into an [ActivityInfo] that contains infos like a localised name.
    ///
    /// # Returns
    /// The respective [ActivityInfo]
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{ActivityInfo, ActivityType, ActivityDataPoints};
    ///
    /// let info = ActivityInfo::from(ActivityType::Basketball);
    /// assert_eq!(info.activity_type, ActivityType::Basketball);
    /// assert_eq!(info.available_data_points, ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION | ActivityDataPoints::HEART_RATE);
    /// assert_eq!(info.id, "basketball");
    /// // assert_eq!(info.name, "Basketball") Assuming your language is set to English, this would work too.
    /// ```
    fn from(activity_type: ActivityType) -> Self {
        match activity_type {
            ActivityType::Basketball => Self::new(
                ActivityType::Basketball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                i18n("Basketball"),
                rgb(245, 194, 17),
            ),
            ActivityType::Bicycling => Self::new(
                ActivityType::Bicycling,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Bicycling"),
                rgb(246, 211, 45),
            ),
            ActivityType::Boxing => Self::new(
                ActivityType::Boxing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                7,
                i18n("Boxing"),
                rgb(192, 28, 40),
            ),
            ActivityType::Dancing => Self::new(
                ActivityType::Dancing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Dancing"),
                rgb(246, 97, 81),
            ),
            ActivityType::Football => Self::new(
                ActivityType::Football,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                3,
                i18n("Football"),
                rgb(152, 106, 68),
            ),
            ActivityType::Golf => Self::new(
                ActivityType::Golf,
                ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION,
                4,
                i18n("Golf"),
                rgb(38, 162, 105),
            ),
            ActivityType::Hiking => Self::new(
                ActivityType::Hiking,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::STEP_COUNT
                    | ActivityDataPoints::DISTANCE,
                8,
                i18n("Hiking"),
                rgb(53, 132, 228),
            ),
            ActivityType::Hockey => Self::new(
                ActivityType::Hockey,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                10,
                i18n("Hockey"),
                rgb(94, 92, 100),
            ),
            ActivityType::HorseRiding => Self::new(
                ActivityType::HorseRiding,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                5,
                i18n("Horse Riding"),
                rgb(145, 65, 172),
            ),
            ActivityType::OtherSports => Self::new(
                ActivityType::OtherSports,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                9,
                i18n("Other Sports"),
                rgb(249, 240, 107),
            ),
            ActivityType::Rollerblading => Self::new(
                ActivityType::Rollerblading,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Rollerblading"),
                rgb(220, 138, 221),
            ),
            ActivityType::Running => Self::new(
                ActivityType::Running,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                15,
                i18n("Running"),
                rgb(255, 190, 111),
            ),
            ActivityType::Skiing => Self::new(
                ActivityType::Skiing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Skiing"),
                rgb(153, 193, 241),
            ),
            ActivityType::Soccer => Self::new(
                ActivityType::Soccer,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Soccer"),
                rgb(143, 240, 164),
            ),
            ActivityType::Softball => Self::new(
                ActivityType::Softball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Softball"),
                rgb(154, 153, 150),
            ),
            ActivityType::Swimming => Self::new(
                ActivityType::Swimming,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Swimming"),
                rgb(26, 95, 180),
            ),
            ActivityType::Tennis => Self::new(
                ActivityType::Tennis,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                i18n("Tennis"),
                rgb(46, 194, 126),
            ),
            ActivityType::TrackAndField => Self::new(
                ActivityType::TrackAndField,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Track And Field"),
                rgb(205, 171, 143),
            ),
            ActivityType::Volleyball => Self::new(
                ActivityType::Volleyball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                5,
                i18n("Volleyball"),
                rgb(222, 221, 218),
            ),
            ActivityType::Walking => Self::new(
                ActivityType::Walking,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                5,
                i18n("Walking"),
                rgb(99, 69, 44),
            ),
        }
    }
}

impl TryFrom<&str> for ActivityInfo {
    type Error = strum::ParseError;

    /// Try converting from an [ActivityInfo] `ID` to an [ActivityInfo].
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{ActivityInfo, ActivityType, ActivityDataPoints};
    /// use std::convert::TryFrom;
    ///
    /// let info = ActivityInfo::try_from("basketball").unwrap();
    /// assert_eq!(info.activity_type, ActivityType::Basketball);
    /// assert_eq!(info.available_data_points, ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION | ActivityDataPoints::HEART_RATE);
    /// assert_eq!(info.id, "basketball");
    /// // assert_eq!(info.name, "Basketball") Assuming your language is set to English, this would work too.
    ///
    /// assert!(ActivityInfo::try_from("unknown").is_err());
    /// ```
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match ActivityType::from_str(val) {
            Ok(t) => Ok(Self::from(t)),
            Err(e) => Err(e),
        }
    }
}

impl ActivityInfo {
    pub fn new(
        activity_type: ActivityType,
        available_data_points: ActivityDataPoints,
        average_calories_burned_per_minute: u32,
        name: String,
        color: RGBA,
    ) -> Self {
        Self {
            id: activity_type.into(),
            activity_type,
            available_data_points,
            average_calories_burned_per_minute,
            name,
            color,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_all_activity_infos() -> Vec<ActivityInfo> {
        let mut v = Vec::with_capacity(20);
        for t in [
            ActivityType::Basketball,
            ActivityType::Bicycling,
            ActivityType::Boxing,
            ActivityType::Dancing,
            ActivityType::Football,
            ActivityType::Golf,
            ActivityType::Hiking,
            ActivityType::Hockey,
            ActivityType::HorseRiding,
            ActivityType::OtherSports,
            ActivityType::Rollerblading,
            ActivityType::Running,
            ActivityType::Skiing,
            ActivityType::Soccer,
            ActivityType::Softball,
            ActivityType::Swimming,
            ActivityType::Tennis,
            ActivityType::TrackAndField,
            ActivityType::Volleyball,
            ActivityType::Walking,
        ] {
            v.push(ActivityInfo::from(t));
        }

        v
    }

    #[test]
    fn non_zero_calories_burned() {
        for info in get_all_activity_infos() {
            assert_ne!(
                info.average_calories_burned_per_minute, 0,
                "Average calories burned must not be 0 to avoid division by zero!"
            );
        }
    }

    #[test]
    fn no_same_color() {
        let all_infos = get_all_activity_infos();
        for info in &all_infos {
            for secondary in &all_infos {
                if info.activity_type != secondary.activity_type {
                    assert_ne!(
                        info.color, secondary.color,
                        "Found duplicate color: {:?} and {:?}",
                        info, secondary
                    )
                }
            }
        }
    }
}
