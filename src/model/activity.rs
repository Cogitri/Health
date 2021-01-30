use crate::{imp_getter_setter, model::ActivityType};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use gdk::subclass::prelude::ObjectSubclass;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uom::si::f32::Length;

mod imp {
    use super::*;
    use crate::{
        inner_refcell_getter_setter,
        model::{ActivityDataPoints, ActivityInfo},
        sync::serialize,
    };
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use uom::si::length::meter;

    static BICYCLING_METERS_PER_MINUTE: u32 = 300;
    static HORSE_RIDING_METERS_PER_MINUTE: u32 = 260;
    static ROLLER_BLADING_METERS_PER_MINUTE: u32 = 240;
    static RUNNING_METERS_PER_MINUTE: u32 = 200;
    static SKIING_METERS_PER_MINUTE: u32 = 400;
    static SWIMMING_METERS_PER_MINUTE: u32 = 160;
    static WALKING_METERS_PER_MINUTE: u32 = 90;

    #[derive(Debug, Deserialize, Serialize)]
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

    impl Activity {
        pub fn autofill_from_calories(&self) {
            let (calories, info) = {
                let inner = self.inner.borrow();
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
                self.inner.borrow_mut().duration =
                    Duration::minutes((calories / info.average_calories_burned_per_minute).into());

                self.autofill_from_minutes();
            }
        }

        pub fn autofill_from_minutes(&self) {
            let mut inner = self.inner.borrow_mut();
            let info = ActivityInfo::from(inner.activity_type.clone());
            let minutes = inner.duration.num_minutes() as u32;

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
                .map(|v| Length::new::<meter>(v as f32))
                {
                    inner.distance = Some(distance);
                }

                match inner.activity_type {
                    ActivityType::Walking | ActivityType::Hiking => {
                        inner.steps = Some(minutes * 100)
                    }
                    ActivityType::Running => inner.steps = Some(minutes * 150),
                    _ => {}
                }
            }
        }

        pub fn autofill_from_distance(&self) {
            let mut inner = self.inner.borrow_mut();
            let info = ActivityInfo::from(inner.activity_type.clone());
            let distance = inner.distance.map(|l| l.get::<meter>()).unwrap_or(0.0) as u32;

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
                    ActivityType::RollerBlading => {
                        Some(distance / ROLLER_BLADING_METERS_PER_MINUTE)
                    }
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
                    inner.duration.num_minutes() as u32 * info.average_calories_burned_per_minute,
                );

                match inner.activity_type {
                    ActivityType::Walking | ActivityType::Hiking | ActivityType::Running => {
                        inner.steps = Some((distance as f32 * 1.4) as u32);
                    }
                    _ => {}
                }
            }
        }

        pub fn autofill_from_steps(&self) {
            let mut inner = self.inner.borrow_mut();
            let info = ActivityInfo::from(inner.activity_type.clone());
            let steps = inner.steps.unwrap_or(0);

            if steps != 0
                && info
                    .available_data_points
                    .contains(ActivityDataPoints::STEP_COUNT)
            {
                match inner.activity_type {
                    ActivityType::Walking => {
                        inner.duration = Duration::minutes((steps / 100) as i64);
                        inner.distance = Some(Length::new::<meter>(
                            (inner.duration.num_minutes() as u32 * WALKING_METERS_PER_MINUTE)
                                as f32,
                        ));
                    }
                    ActivityType::Hiking => {
                        inner.duration = Duration::minutes((steps / 80) as i64);
                        inner.distance = Some(Length::new::<meter>(
                            (inner.duration.num_minutes() as u32 * WALKING_METERS_PER_MINUTE)
                                as f32,
                        ));
                    }
                    ActivityType::Running => {
                        inner.duration = Duration::minutes((steps / 150) as i64);
                        inner.distance = Some(Length::new::<meter>(
                            (inner.duration.num_minutes() as u32 * RUNNING_METERS_PER_MINUTE)
                                as f32,
                        ));
                    }
                    _ => {}
                }

                inner.calories_burned = Some(
                    info.average_calories_burned_per_minute * inner.duration.num_minutes() as u32,
                );
            }
        }

        inner_refcell_getter_setter!(activity_type, ActivityType);
        inner_refcell_getter_setter!(calories_burned, Option<u32>);
        inner_refcell_getter_setter!(date, DateTime<FixedOffset>);
        inner_refcell_getter_setter!(distance, Option<Length>);
        inner_refcell_getter_setter!(heart_rate_avg, Option<u32>);
        inner_refcell_getter_setter!(heart_rate_max, Option<u32>);
        inner_refcell_getter_setter!(heart_rate_min, Option<u32>);
        inner_refcell_getter_setter!(duration, Duration);
        inner_refcell_getter_setter!(steps, Option<u32>);
    }
}

glib::wrapper! {
    pub struct Activity(ObjectSubclass<imp::Activity>);
}

impl Activity {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create Activity")
    }

    fn get_priv(&self) -> &imp::Activity {
        imp::Activity::from_instance(self)
    }

    pub fn autofill_from_calories(&self) {
        self.get_priv().autofill_from_calories();
    }

    pub fn autofill_from_distance(&self) {
        self.get_priv().autofill_from_distance();
    }

    pub fn autofill_from_minutes(&self) {
        self.get_priv().autofill_from_minutes();
    }

    pub fn autofill_from_steps(&self) {
        self.get_priv().autofill_from_steps();
    }

    imp_getter_setter!(activity_type, ActivityType);
    imp_getter_setter!(calories_burned, Option<u32>);
    imp_getter_setter!(date, DateTime<FixedOffset>);
    imp_getter_setter!(distance, Option<Length>);
    imp_getter_setter!(heart_rate_avg, Option<u32>);
    imp_getter_setter!(heart_rate_max, Option<u32>);
    imp_getter_setter!(heart_rate_min, Option<u32>);
    imp_getter_setter!(duration, Duration);
    imp_getter_setter!(steps, Option<u32>);
}

impl Serialize for Activity {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.get_priv().inner.borrow().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Activity {
    fn deserialize<D>(deserializer: D) -> Result<Activity, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = imp::ActivityMut::deserialize(deserializer)?;

        let a = Activity::new();
        imp::Activity::from_instance(&a).inner.replace(inner);
        Ok(a)
    }
}
