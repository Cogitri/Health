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
use gtk::glib::{self, GBoxed};
use std::{convert::TryFrom, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, GBoxed)]
#[gboxed(type_name = "ActivityInfo")]
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

impl From<ActivityType> for ActivityInfo {
    /// Converts an [ActivityType] into an [ActivityInfo] that contains infos like a localised name.
    ///
    /// # Returns
    /// The respective [ActivityInfo]
    ///
    /// # Examples
    /// ```
    /// use libhealth::{ActivityInfo, ActivityType, ActivityDataPoints};
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
                RGBA {
                    red: 0.0,
                    green: 0.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Bicycling => Self::new(
                ActivityType::Bicycling,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Bicycling"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Boxing => Self::new(
                ActivityType::Boxing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                7,
                i18n("Boxing"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Dancing => Self::new(
                ActivityType::Dancing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Dancing"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Football => Self::new(
                ActivityType::Football,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                3,
                i18n("Football"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Golf => Self::new(
                ActivityType::Golf,
                ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION,
                4,
                i18n("Golf"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
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
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Hockey => Self::new(
                ActivityType::Hockey,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                10,
                i18n("Hockey"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::HorseRiding => Self::new(
                ActivityType::HorseRiding,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                5,
                i18n("Horse Riding"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::OtherSports => Self::new(
                ActivityType::OtherSports,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                9,
                i18n("Other Sports"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Rollerblading => Self::new(
                ActivityType::Rollerblading,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Rollerblading"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
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
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Skiing => Self::new(
                ActivityType::Skiing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Skiing"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Soccer => Self::new(
                ActivityType::Soccer,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Soccer"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Softball => Self::new(
                ActivityType::Softball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Softball"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Swimming => Self::new(
                ActivityType::Swimming,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Swimming"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::Tennis => Self::new(
                ActivityType::Tennis,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                i18n("Tennis"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            ),
            ActivityType::TrackAndField => Self::new(
                ActivityType::TrackAndField,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Track And Field"),
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
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
                RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
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
                RGBA {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
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
    /// use libhealth::{ActivityInfo, ActivityType, ActivityDataPoints};
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
            id: activity_type.clone().into(),
            activity_type,
            available_data_points,
            average_calories_burned_per_minute,
            name,
            color,
        }
    }
}
