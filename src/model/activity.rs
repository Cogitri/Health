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
    prelude::*,
};
use chrono::{DateTime, Duration, FixedOffset, Local};
use gtk::glib::{self, prelude::*, subclass::prelude::*};
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};
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
    use crate::{model::ActivityType, prelude::*, sync::serialize};
    use chrono::{DateTime, Duration, FixedOffset, Utc};
    use gtk::{glib, prelude::*, subclass::prelude::*};
    use std::{cell::RefCell, convert::TryInto, str::FromStr};
    use uom::si::{f32::Length, length::meter};

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

    impl ObjectImpl for Activity {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;

            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "activity-type",
                        "activity-type",
                        "activity-type",
                        Some("walking"),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "calories-burned",
                        "calories-burned",
                        "calories-burned",
                        -1,
                        u32::MAX.into(),
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecBoxed::new(
                        "date",
                        "date",
                        "date",
                        DateTimeBoxed::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecFloat::new(
                        "distance-meter",
                        "distance-meter",
                        "distance-meter",
                        -1.0,
                        f32::MAX,
                        -1.0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "duration-seconds",
                        "duration-seconds",
                        "duration-seconds",
                        i64::MIN,
                        i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "heart-rate-avg",
                        "heart-rate-avg",
                        "heart-rate-avg",
                        -1,
                        u32::MAX.into(),
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "heart-rate-max",
                        "heart-rate-max",
                        "heart-rate-max",
                        -1,
                        u32::MAX.into(),
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "heart-rate-min",
                        "heart-rate-min",
                        "heart-rate-min",
                        -1,
                        u32::MAX.into(),
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecInt64::new(
                        "steps",
                        "steps",
                        "steps",
                        -1,
                        u32::MAX.into(),
                        -1,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "activity-type" => {
                    self.inner.borrow_mut().activity_type =
                        ActivityType::from_str(&value.get::<String>().unwrap()).unwrap()
                }
                "calories-burned" => {
                    self.inner.borrow_mut().calories_burned =
                        value.get::<i64>().unwrap().try_into().ok();
                }
                "date" => {
                    self.inner.borrow_mut().date = value.get::<DateTimeBoxed>().unwrap().0;
                }
                "distance-meter" => {
                    let value = value.get::<f32>().unwrap();
                    if value < 0.0 {
                        self.inner.borrow_mut().distance = None;
                    } else {
                        self.inner.borrow_mut().distance = Some(Length::new::<meter>(value));
                    }
                }
                "duration-seconds" => {
                    self.inner.borrow_mut().duration = Duration::seconds(value.get().unwrap());
                }
                "heart-rate-avg" => {
                    self.inner.borrow_mut().heart_rate_avg =
                        value.get::<i64>().unwrap().try_into().ok();
                }
                "heart-rate-max" => {
                    self.inner.borrow_mut().heart_rate_max =
                        value.get::<i64>().unwrap().try_into().ok();
                }
                "heart-rate-min" => {
                    self.inner.borrow_mut().heart_rate_min =
                        value.get::<i64>().unwrap().try_into().ok();
                }
                "steps" => {
                    self.inner.borrow_mut().steps = value.get::<i64>().unwrap().try_into().ok();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "activity-type" => self.inner.borrow().activity_type.to_value(),
                "calories-burned" => self
                    .inner
                    .borrow()
                    .calories_burned
                    .unwrap_ori(-1)
                    .to_value(),
                "date" => DateTimeBoxed(self.inner.borrow().date).to_value(),
                "distance-meter" => self
                    .inner
                    .borrow()
                    .distance
                    .map_or(-1.0, |d| d.get::<meter>())
                    .to_value(),
                "duration-seconds" => self.inner.borrow().duration.num_seconds().to_value(),
                "heart-rate-avg" => self.inner.borrow().heart_rate_avg.unwrap_ori(-1).to_value(),
                "heart-rate-max" => self.inner.borrow().heart_rate_max.unwrap_ori(-1).to_value(),
                "heart-rate-min" => self.inner.borrow().heart_rate_min.unwrap_ori(-1).to_value(),
                "steps" => self.inner.borrow().steps.unwrap_ori(-1).to_value(),
                _ => unimplemented!(),
            }
        }
    }
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
        let imp = self.imp();

        let (calories, info) = {
            let inner = imp.inner.borrow();
            (
                inner.calories_burned.unwrap_or(0),
                ActivityInfo::from(inner.activity_type),
            )
        };

        if calories != 0
            && info
                .available_data_points
                .contains(ActivityDataPoints::CALORIES_BURNED)
        {
            imp.inner.borrow_mut().duration =
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
        let imp = self.imp();

        let mut inner = imp.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type);
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
        let imp = self.imp();

        let mut inner = imp.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type);
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
        let imp = self.imp();

        let mut inner = imp.inner.borrow_mut();
        let info = ActivityInfo::from(inner.activity_type);
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
        glib::Object::new(&[("date", &DateTimeBoxed(Local::now().into()))])
            .expect("Failed to create Activity")
    }

    pub fn activity_type(&self) -> ActivityType {
        ActivityType::from_str(&self.property::<String>("activity-type")).unwrap()
    }

    pub fn calories_burned(&self) -> Option<u32> {
        self.property::<i64>("calories-burned").try_into().ok()
    }

    pub fn date(&self) -> DateTime<FixedOffset> {
        self.property::<DateTimeBoxed>("date").0
    }

    pub fn distance(&self) -> Option<Length> {
        let value = self.property::<f32>("distance-meter");
        if value < 0.0 {
            None
        } else {
            Some(Length::new::<meter>(value))
        }
    }

    pub fn duration(&self) -> Duration {
        Duration::seconds(self.property("duration-seconds"))
    }

    pub fn heart_rate_avg(&self) -> Option<u32> {
        self.property::<i64>("heart-rate-avg").try_into().ok()
    }

    pub fn heart_rate_max(&self) -> Option<u32> {
        self.property::<i64>("heart-rate-max").try_into().ok()
    }

    pub fn heart_rate_min(&self) -> Option<u32> {
        self.property::<i64>("heart-rate-min").try_into().ok()
    }

    pub fn steps(&self) -> Option<u32> {
        self.property::<i64>("steps").try_into().ok()
    }

    pub fn set_activity_type(&self, value: ActivityType) -> &Self {
        self.set_property("activity-type", value);
        self
    }

    pub fn set_calories_burned(&self, value: Option<u32>) -> &Self {
        self.set_property("calories-burned", value.unwrap_ori(-1));
        self
    }

    pub fn set_date(&self, value: DateTime<FixedOffset>) -> &Self {
        self.set_property("date", DateTimeBoxed(value));
        self
    }

    pub fn set_distance(&self, value: Option<Length>) -> &Self {
        self.set_property("distance-meter", value.map_or(-1.0, |v| v.get::<meter>()));
        self
    }

    pub fn set_duration(&self, value: Duration) -> &Self {
        self.set_property("duration-seconds", value.num_seconds());
        self
    }

    pub fn set_heart_rate_avg(&self, value: Option<u32>) -> &Self {
        self.set_property("heart-rate-avg", value.unwrap_ori(-1));
        self
    }

    pub fn set_heart_rate_max(&self, value: Option<u32>) -> &Self {
        self.set_property("heart-rate-max", value.unwrap_ori(-1));
        self
    }

    pub fn set_heart_rate_min(&self, value: Option<u32>) -> &Self {
        self.set_property("heart-rate-min", value.unwrap_ori(-1));
        self
    }

    pub fn set_steps(&self, value: Option<u32>) -> &Self {
        self.set_property("steps", value.unwrap_ori(-1));
        self
    }
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
