/* activity.rs
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
    model::{ActivityDataPoints, ActivityInfo, ActivityType},
    refcell_getter_setter,
};
use chrono::{DateTime, Duration, FixedOffset};
use gtk::glib::{self, subclass::prelude::*};
use std::convert::TryFrom;
use uom::si::{f32::Length, length::meter};

static BICYCLING_METERS_PER_MINUTE: u32 = 300;
static HORSE_RIDING_METERS_PER_MINUTE: u32 = 260;
static ROLLER_BLADING_METERS_PER_MINUTE: u32 = 240;
static RUNNING_METERS_PER_MINUTE: u32 = 200;
static SKIING_METERS_PER_MINUTE: u32 = 400;
static SWIMMING_METERS_PER_MINUTE: u32 = 160;
static WALKING_METERS_PER_MINUTE: u32 = 90;

static WALKING_STEPS_PER_MINUTE: u32 = 100;
static RUNNING_STEPS_PER_MINUTE: u32 = 150;

mod imp {
    use crate::{model::ActivityType, sync::serialize};
    use chrono::{DateTime, Duration, FixedOffset, Utc};
    use gtk::{glib, subclass::prelude::*};
    use std::cell::RefCell;
    use uom::si::f32::Length;

    #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
    pub struct ActivityMut {
        #[serde(serialize_with = "serialize::serialize_activity_type")]
        #[serde(deserialize_with = "serialize::deserialize_activity_type")]
        pub activity_type: ActivityType,
        pub calories_burned: Option<u32>,
        #[serde(serialize_with = "serialize::serialize_date")]
        #[serde(deserialize_with = "serialize::deserialize_date")]
        pub date: DateTime<FixedOffset>,
        #[serde(serialize_with = "serialize::serialize_distance")]
        #[serde(deserialize_with = "serialize::deserialize_distance")]
        pub distance: Option<Length>,
        pub heart_rate_avg: Option<u32>,
        pub heart_rate_max: Option<u32>,
        pub heart_rate_min: Option<u32>,
        #[serde(serialize_with = "serialize::serialize_duration")]
        #[serde(deserialize_with = "serialize::deserialize_duration")]
        pub duration: Duration,
        pub steps: Option<u32>,
    }

    pub struct Activity {
        pub inner: RefCell<ActivityMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Activity {
        const NAME: &'static str = "HealthActivity";
        type ParentType = glib::Object;
        type Type = super::Activity;

        fn new() -> Self {
            Self {
                inner: RefCell::new(ActivityMut {
                    activity_type: ActivityType::default(),
                    calories_burned: None,
                    date: Utc::now().into(),
                    distance: None,
                    heart_rate_avg: None,
                    heart_rate_max: None,
                    heart_rate_min: None,
                    duration: Duration::seconds(0),
                    steps: None,
                }),
            }
        }
    }

    impl ObjectImpl for Activity {}
}

glib::wrapper! {
    /// An [Activity] represents a single activity an user has performed (e.g. walking).
    pub struct Activity(ObjectSubclass<imp::Activity>);
}

impl Default for Activity {
    fn default() -> Self {
        Self::new()
    }
}

impl Activity {
    /// Try interpolating data from the `calories` that are set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{Activity, ActivityType};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_calories_burned(Some(100));
    /// activity.autofill_from_calories();
    /// assert_eq!(activity.duration().num_minutes(), 20);
    /// ```
    pub fn autofill_from_calories(&self) {
        let self_ = self.imp();

        let (calories, info) = {
            let inner = self_.inner.borrow();
            (
                inner.calories_burned.unwrap_or(0),
                ActivityInfo::from(inner.activity_type.clone()),
            )
        };

        if calories != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::CALORIES_BURNED)
        {
            self_.inner.borrow_mut().duration =
                Duration::minutes((calories / info.average_calories_burned_per_minute).into());

            self.autofill_from_minutes();
        }
    }

    /// Try interpolating data from the `distance` that is set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{Activity, ActivityType};
    /// use uom::si::{f32::Length, length::kilometer};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_distance(Some(Length::new::<kilometer>(1.0)));
    /// activity.autofill_from_distance();
    /// assert_eq!(activity.duration().num_minutes(), 11);
    /// ```
    pub fn autofill_from_distance(&self) {
        let self_ = self.imp();

        let mut inner = self_.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type.clone());
        let distance = inner.distance.map_or(0.0, |l| l.get::<meter>()) as u32;

        if distance != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::DISTANCE)
        {
            if let Some(duration) = match inner.activity_type {
                ActivityType::Bicycling => Some(distance / BICYCLING_METERS_PER_MINUTE),
                ActivityType::HorseRiding => Some(distance / HORSE_RIDING_METERS_PER_MINUTE),
                ActivityType::Hiking | ActivityType::Walking => {
                    Some(distance / WALKING_METERS_PER_MINUTE)
                }
                ActivityType::Rollerblading => Some(distance / ROLLER_BLADING_METERS_PER_MINUTE),
                ActivityType::Running | ActivityType::TrackAndField => {
                    Some(distance / RUNNING_METERS_PER_MINUTE)
                }
                ActivityType::Skiing => Some(distance / SKIING_METERS_PER_MINUTE),
                ActivityType::Swimming => Some(distance / SWIMMING_METERS_PER_MINUTE),
                _ => None,
            }
            .map(|v| Duration::minutes(v.into()))
            {
                inner.duration = duration;
            }

            inner.calories_burned = Some(
                u32::try_from(inner.duration.num_minutes()).unwrap()
                    * info.average_calories_burned_per_minute,
            );

            match inner.activity_type {
                ActivityType::Walking | ActivityType::Hiking => {
                    inner.steps = Some(
                        u32::try_from(inner.duration.num_minutes()).unwrap()
                            * WALKING_STEPS_PER_MINUTE,
                    )
                }
                ActivityType::Running => {
                    inner.steps = Some(
                        u32::try_from(inner.duration.num_minutes()).unwrap()
                            * RUNNING_STEPS_PER_MINUTE,
                    )
                }
                _ => {}
            }
        }
    }

    /// Try interpolating data from the `minutes` that is set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{Activity, ActivityType};
    /// use chrono::Duration;
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_duration(Duration::minutes(20));
    /// activity.autofill_from_minutes();
    /// assert_eq!(activity.calories_burned(), Some(100));
    /// ```
    pub fn autofill_from_minutes(&self) {
        let self_ = self.imp();

        let mut inner = self_.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type.clone());
        let minutes = u32::try_from(inner.duration.num_minutes()).unwrap();

        if minutes != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::DURATION)
        {
            inner.calories_burned = Some(info.average_calories_burned_per_minute * minutes);

            if let Some(distance) = match inner.activity_type {
                ActivityType::Bicycling => Some(BICYCLING_METERS_PER_MINUTE * minutes),
                ActivityType::HorseRiding => Some(HORSE_RIDING_METERS_PER_MINUTE * minutes),
                ActivityType::Hiking | ActivityType::Walking => {
                    Some(WALKING_METERS_PER_MINUTE * minutes)
                }
                ActivityType::Rollerblading => Some(ROLLER_BLADING_METERS_PER_MINUTE * minutes),
                ActivityType::Running | ActivityType::TrackAndField => {
                    Some(RUNNING_METERS_PER_MINUTE * minutes)
                }
                ActivityType::Skiing => Some(SKIING_METERS_PER_MINUTE * minutes),
                ActivityType::Swimming => Some(SWIMMING_METERS_PER_MINUTE * minutes),
                _ => None,
            }
            .map(|v: u32| Length::new::<meter>(v as f32))
            {
                inner.distance = Some(distance);
            }

            match inner.activity_type {
                ActivityType::Walking | ActivityType::Hiking => {
                    inner.steps = Some(minutes * WALKING_STEPS_PER_MINUTE)
                }
                ActivityType::Running => inner.steps = Some(minutes * RUNNING_STEPS_PER_MINUTE),
                _ => {}
            }
        }
    }

    /// Try interpolating data from the `steps` that is set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::model::{Activity, ActivityType};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_steps(Some(100));
    /// activity.autofill_from_steps();
    /// assert_eq!(activity.duration().num_minutes(), 1);
    /// ```
    pub fn autofill_from_steps(&self) {
        let self_ = self.imp();

        let mut inner = self_.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type.clone());
        let steps = inner.steps.unwrap_or(0);

        if steps != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::STEP_COUNT)
        {
            match inner.activity_type {
                ActivityType::Walking | ActivityType::Hiking => {
                    inner.duration = Duration::minutes((steps / WALKING_STEPS_PER_MINUTE).into());
                    inner.distance = Some(Length::new::<meter>(
                        (u32::try_from(inner.duration.num_minutes()).unwrap()
                            * WALKING_METERS_PER_MINUTE) as f32,
                    ));
                }
                ActivityType::Running => {
                    inner.duration = Duration::minutes((steps / RUNNING_STEPS_PER_MINUTE).into());
                    inner.distance = Some(Length::new::<meter>(
                        (u32::try_from(inner.duration.num_minutes()).unwrap()
                            * RUNNING_METERS_PER_MINUTE) as f32,
                    ));
                }
                _ => {}
            }

            inner.calories_burned = Some(
                info.average_calories_burned_per_minute
                    * u32::try_from(inner.duration.num_minutes()).unwrap(),
            );
        }
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Activity")
    }

    fn imp(&self) -> &imp::Activity {
        imp::Activity::from_instance(self)
    }

    refcell_getter_setter!(activity_type, ActivityType);
    refcell_getter_setter!(calories_burned, Option<u32>);
    refcell_getter_setter!(date, DateTime<FixedOffset>);
    refcell_getter_setter!(distance, Option<Length>);
    refcell_getter_setter!(heart_rate_avg, Option<u32>);
    refcell_getter_setter!(heart_rate_max, Option<u32>);
    refcell_getter_setter!(heart_rate_min, Option<u32>);
    refcell_getter_setter!(duration, Duration);
    refcell_getter_setter!(steps, Option<u32>);
}

impl serde::Serialize for Activity {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        self.imp().inner.borrow().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Activity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = imp::ActivityMut::deserialize(deserializer)?;

        let a = Self::new();
        a.imp().inner.replace(inner);
        Ok(a)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{DateTime, Duration};
    use serde_test::{assert_ser_tokens, Token};
    use uom::si::{f32::Length, length::kilometer};

    #[test]
    fn new() {
        Activity::new();
    }

    #[test]
    fn serialize_default() {
        let a = Activity::new();
        a.set_date(DateTime::parse_from_rfc3339("2021-03-28T20:39:08.315749637+00:00").unwrap());
        assert_ser_tokens(
            &a,
            &[
                Token::Struct {
                    name: "ActivityMut",
                    len: 9,
                },
                Token::Str("activity_type"),
                Token::Str("walking"),
                Token::Str("calories_burned"),
                Token::None,
                Token::Str("date"),
                Token::Str("2021-03-28T20:39:08.315749637+00:00"),
                Token::Str("distance"),
                Token::F32(0.0),
                Token::Str("heart_rate_avg"),
                Token::None,
                Token::Str("heart_rate_max"),
                Token::None,
                Token::Str("heart_rate_min"),
                Token::None,
                Token::Str("duration"),
                Token::U32(0),
                Token::Str("steps"),
                Token::None,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn autofill_from_steps() {
        let a = Activity::new();
        a.set_steps(Some(2000));
        a.autofill_from_steps();
        assert_eq!(a.duration(), Duration::minutes(20));
        assert_eq!(a.distance(), Some(Length::new::<kilometer>(1.8)));
        assert_eq!(a.calories_burned(), Some(100));
    }

    #[test]
    fn autofill_from_minutes() {
        let a = Activity::new();
        a.set_duration(Duration::minutes(20));
        a.autofill_from_minutes();
        assert_eq!(a.calories_burned(), Some(100));
        assert_eq!(a.steps(), Some(2000));
        assert_eq!(a.distance(), Some(Length::new::<kilometer>(1.8)));
    }

    #[test]
    fn autofill_from_distance() {
        let a = Activity::new();
        a.set_distance(Some(Length::new::<kilometer>(1.8)));
        a.autofill_from_distance();
        assert_eq!(a.calories_burned(), Some(100));
        assert_eq!(a.steps(), Some(2000));
        assert_eq!(a.duration(), Duration::minutes(20));
    }

    #[test]
    fn autofill_from_calories() {
        let a = Activity::new();
        a.set_calories_burned(Some(100));
        a.autofill_from_calories();
        assert_eq!(a.distance(), Some(Length::new::<kilometer>(1.8)));
        assert_eq!(a.steps(), Some(2000));
        assert_eq!(a.duration(), Duration::minutes(20));
    }

    #[test]
    fn non_zero() {
        assert_ne!(BICYCLING_METERS_PER_MINUTE, 0);
        assert_ne!(HORSE_RIDING_METERS_PER_MINUTE, 0);
        assert_ne!(ROLLER_BLADING_METERS_PER_MINUTE, 0);
        assert_ne!(RUNNING_METERS_PER_MINUTE, 0);
        assert_ne!(SKIING_METERS_PER_MINUTE, 0);
        assert_ne!(SWIMMING_METERS_PER_MINUTE, 0);
        assert_ne!(WALKING_METERS_PER_MINUTE, 0);
        assert_ne!(WALKING_STEPS_PER_MINUTE, 0);
        assert_ne!(RUNNING_STEPS_PER_MINUTE, 0);
    }
}
