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
use std::{convert::TryFrom, str::FromStr};

/// A struct containing informations about a certain activity type.
#[derive(Debug, Clone)]
pub struct ActivityInfo {
    pub activity_type: ActivityType,
    pub available_data_points: ActivityDataPoints,
    pub average_calories_burned_per_minute: u32,
    /// An ID that can be used for saving things to the DB.
    pub id: &'static str,
    /// The localised name of the `ActivityType`.
    pub name: String,
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
            ActivityType::Basketball => ActivityInfo::new(
                ActivityType::Basketball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                i18n("Basketball"),
            ),
            ActivityType::Bicycling => ActivityInfo::new(
                ActivityType::Bicycling,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Bicycling"),
            ),
            ActivityType::Boxing => ActivityInfo::new(
                ActivityType::Boxing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                7,
                i18n("Boxing"),
            ),
            ActivityType::Dancing => ActivityInfo::new(
                ActivityType::Dancing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Dancing"),
            ),
            ActivityType::Football => ActivityInfo::new(
                ActivityType::Football,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                3,
                i18n("Football"),
            ),
            ActivityType::Golf => ActivityInfo::new(
                ActivityType::Golf,
                ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION,
                4,
                i18n("Golf"),
            ),
            ActivityType::Hiking => ActivityInfo::new(
                ActivityType::Hiking,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::STEP_COUNT
                    | ActivityDataPoints::DISTANCE,
                8,
                i18n("Hiking"),
            ),
            ActivityType::Hockey => ActivityInfo::new(
                ActivityType::Hockey,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                10,
                i18n("Hockey"),
            ),
            ActivityType::HorseRiding => ActivityInfo::new(
                ActivityType::HorseRiding,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                5,
                i18n("Horse Riding"),
            ),
            ActivityType::OtherSports => ActivityInfo::new(
                ActivityType::OtherSports,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                9,
                i18n("Other Sports"),
            ),
            ActivityType::Rollerblading => ActivityInfo::new(
                ActivityType::Rollerblading,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                i18n("Rollerblading"),
            ),
            ActivityType::Running => ActivityInfo::new(
                ActivityType::Running,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                15,
                i18n("Running"),
            ),
            ActivityType::Skiing => ActivityInfo::new(
                ActivityType::Skiing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Skiing"),
            ),
            ActivityType::Soccer => ActivityInfo::new(
                ActivityType::Soccer,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                i18n("Soccer"),
            ),
            ActivityType::Softball => ActivityInfo::new(
                ActivityType::Softball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Softball"),
            ),
            ActivityType::Swimming => ActivityInfo::new(
                ActivityType::Swimming,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                i18n("Swimming"),
            ),
            ActivityType::Tennis => ActivityInfo::new(
                ActivityType::Tennis,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                i18n("Tennis"),
            ),
            ActivityType::TrackAndField => ActivityInfo::new(
                ActivityType::TrackAndField,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                i18n("Track And Field"),
            ),
            ActivityType::Volleyball => ActivityInfo::new(
                ActivityType::Volleyball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                5,
                i18n("Volleyball"),
            ),
            ActivityType::Walking => ActivityInfo::new(
                ActivityType::Walking,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                5,
                i18n("Walking"),
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
            Ok(t) => Ok(ActivityInfo::from(t)),
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
    ) -> Self {
        Self {
            id: activity_type.clone().into(),
            activity_type,
            available_data_points,
            average_calories_burned_per_minute,
            name,
        }
    }
}
