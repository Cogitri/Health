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
use glib::subclass::prelude::*;
use std::convert::TryFrom;
use uom::si::{f32::Length, length::meter};

static BICYCLING_METERS_PER_MINUTE: u32 = 300;
static HORSE_RIDING_METERS_PER_MINUTE: u32 = 260;
static ROLLER_BLADING_METERS_PER_MINUTE: u32 = 240;
static RUNNING_METERS_PER_MINUTE: u32 = 200;
static SKIING_METERS_PER_MINUTE: u32 = 400;
static SWIMMING_METERS_PER_MINUTE: u32 = 160;
static WALKING_METERS_PER_MINUTE: u32 = 90;

mod imp {
    use crate::{model::ActivityType, sync::serialize};
    use chrono::{DateTime, Duration, FixedOffset, Utc};
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use uom::si::f32::Length;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
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

    impl ObjectSubclass for Activity {
        const NAME: &'static str = "HealthActivity";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::Activity;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(ActivityMut {
                    activity_type: ActivityType::Walking,
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

impl Activity {
    /// Try interpolating data from the `calories` that are set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::{Activity, ActivityType};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_calories_burned(Some(100));
    /// activity.autofill_from_calories();
    /// assert_eq!(activity.get_duration().num_minutes(), 20);
    /// ```
    pub fn autofill_from_calories(&self) {
        let self_ = self.get_priv();

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
    /// use libhealth::{Activity, ActivityType};
    /// use uom::si::{f32::Length, length::kilometer};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_distance(Some(Length::new::<kilometer>(1.0)));
    /// activity.autofill_from_distance();
    /// assert_eq!(activity.get_duration().num_minutes(), 11);
    /// ```
    pub fn autofill_from_distance(&self) {
        let self_ = self.get_priv();

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
                ActivityType::RollerBlading => Some(distance / ROLLER_BLADING_METERS_PER_MINUTE),
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
                ActivityType::Walking | ActivityType::Hiking | ActivityType::Running => {
                    inner.steps = Some((distance as f32 * 1.4) as u32);
                }
                _ => {}
            }
        }
    }

    /// Try interpolating data from the `minutes` that is set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::{Activity, ActivityType};
    /// use chrono::Duration;
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_duration(Duration::minutes(20));
    /// activity.autofill_from_minutes();
    /// assert_eq!(activity.get_calories_burned(), Some(100));
    /// ```
    pub fn autofill_from_minutes(&self) {
        let self_ = self.get_priv();

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
                ActivityType::RollerBlading => Some(ROLLER_BLADING_METERS_PER_MINUTE * minutes),
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
                ActivityType::Walking | ActivityType::Hiking => inner.steps = Some(minutes * 100),
                ActivityType::Running => inner.steps = Some(minutes * 150),
                _ => {}
            }
        }
    }

    /// Try interpolating data from the `steps` that is set on `self`.
    ///
    /// # Examples
    /// ```
    /// use libhealth::{Activity, ActivityType};
    ///
    /// let activity = Activity::new();
    /// activity.set_activity_type(ActivityType::Walking);
    /// activity.set_steps(Some(100));
    /// activity.autofill_from_steps();
    /// assert_eq!(activity.get_duration().num_minutes(), 1);
    /// ```
    pub fn autofill_from_steps(&self) {
        let self_ = self.get_priv();

        let mut inner = self_.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type.clone());
        let steps = inner.steps.unwrap_or(0);
        let num_minutes = u32::try_from(inner.duration.num_minutes()).unwrap();

        if steps != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::STEP_COUNT)
        {
            match inner.activity_type {
                ActivityType::Walking => {
                    inner.duration = Duration::minutes((steps / 100).into());
                    inner.distance = Some(Length::new::<meter>(
                        (num_minutes * WALKING_METERS_PER_MINUTE) as f32,
                    ));
                }
                ActivityType::Hiking => {
                    inner.duration = Duration::minutes((steps / 80).into());
                    inner.distance = Some(Length::new::<meter>(
                        (num_minutes * WALKING_METERS_PER_MINUTE) as f32,
                    ));
                }
                ActivityType::Running => {
                    inner.duration = Duration::minutes((steps / 150).into());
                    inner.distance = Some(Length::new::<meter>(
                        (num_minutes * RUNNING_METERS_PER_MINUTE) as f32,
                    ));
                }
                _ => {}
            }

            inner.calories_burned = Some(info.average_calories_burned_per_minute * num_minutes);
        }
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Activity")
    }

    fn get_priv(&self) -> &imp::Activity {
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
        self.get_priv().inner.borrow().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Activity {
    fn deserialize<D>(deserializer: D) -> Result<Activity, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = imp::ActivityMut::deserialize(deserializer)?;

        let a = Activity::new();
        a.get_priv().inner.replace(inner);
        Ok(a)
    }
}
