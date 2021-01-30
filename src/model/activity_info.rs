use crate::{
    core::i18n,
    model::{ActivityDataPoints, ActivityType},
};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct ActivityInfo {
    pub activity_type: ActivityType,
    pub available_data_points: ActivityDataPoints,
    pub average_calories_burned_per_minute: u32,
    pub id: &'static str,
    pub name: String,
}

impl From<ActivityType> for ActivityInfo {
    fn from(activity_type: ActivityType) -> Self {
        match activity_type {
            ActivityType::Basketball => ActivityInfo::new(
                ActivityType::Basketball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                "basketball",
                i18n("Basketball"),
            ),
            ActivityType::Bicycling => ActivityInfo::new(
                ActivityType::Bicycling,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                "bicycling",
                i18n("Bicycling"),
            ),
            ActivityType::Boxing => ActivityInfo::new(
                ActivityType::Boxing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                7,
                "boxing",
                i18n("Boxing"),
            ),
            ActivityType::Dancing => ActivityInfo::new(
                ActivityType::Dancing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                "dancing",
                i18n("Dancing"),
            ),
            ActivityType::Football => ActivityInfo::new(
                ActivityType::Football,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                3,
                "football",
                i18n("Football"),
            ),
            ActivityType::Golf => ActivityInfo::new(
                ActivityType::Golf,
                ActivityDataPoints::CALORIES_BURNED | ActivityDataPoints::DURATION,
                4,
                "golf",
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
                "hiking",
                i18n("Hiking"),
            ),
            ActivityType::Hockey => ActivityInfo::new(
                ActivityType::Hockey,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                10,
                "hockey",
                i18n("Hockey"),
            ),
            ActivityType::HorseRiding => ActivityInfo::new(
                ActivityType::HorseRiding,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                5,
                "horse_riding",
                i18n("Horse Riding"),
            ),
            ActivityType::OtherSports => ActivityInfo::new(
                ActivityType::OtherSports,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                9,
                "other_sports",
                i18n("Other Sports"),
            ),
            ActivityType::RollerBlading => ActivityInfo::new(
                ActivityType::RollerBlading,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                10,
                "rollerblading",
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
                "running",
                i18n("Running"),
            ),
            ActivityType::Skiing => ActivityInfo::new(
                ActivityType::Skiing,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                "skiing",
                i18n("Skiing"),
            ),
            ActivityType::Soccer => ActivityInfo::new(
                ActivityType::Soccer,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                8,
                "soccer",
                i18n("Soccer"),
            ),
            ActivityType::Softball => ActivityInfo::new(
                ActivityType::Softball,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                "softball",
                i18n("Softball"),
            ),
            ActivityType::Swimming => ActivityInfo::new(
                ActivityType::Swimming,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE,
                12,
                "swimming",
                i18n("Swimming"),
            ),
            ActivityType::Tennis => ActivityInfo::new(
                ActivityType::Tennis,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                6,
                "tennis",
                i18n("Tennis"),
            ),
            ActivityType::TrackAndField => ActivityInfo::new(
                ActivityType::TrackAndField,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE,
                5,
                "track_and_field",
                i18n("Track And Field"),
            ),
            ActivityType::VolleyBall => ActivityInfo::new(
                ActivityType::VolleyBall,
                ActivityDataPoints::CALORIES_BURNED
                    | ActivityDataPoints::DURATION
                    | ActivityDataPoints::HEART_RATE
                    | ActivityDataPoints::DISTANCE
                    | ActivityDataPoints::STEP_COUNT,
                5,
                "volleyball",
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
                "walking",
                i18n("Walking"),
            ),
        }
    }
}

impl TryFrom<&str> for ActivityInfo {
    type Error = ();

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val.to_lowercase().as_str() {
            "basketball" => Ok(ActivityInfo::from(ActivityType::Basketball)),
            "bicycling" => Ok(ActivityInfo::from(ActivityType::Bicycling)),
            "boxing" => Ok(ActivityInfo::from(ActivityType::Boxing)),
            "dancing" => Ok(ActivityInfo::from(ActivityType::Dancing)),
            "football" => Ok(ActivityInfo::from(ActivityType::Football)),
            "golf" => Ok(ActivityInfo::from(ActivityType::Golf)),
            "hiking" => Ok(ActivityInfo::from(ActivityType::Hiking)),
            "hockey" => Ok(ActivityInfo::from(ActivityType::Hockey)),
            "horse_riding" => Ok(ActivityInfo::from(ActivityType::HorseRiding)),
            "other_sports" => Ok(ActivityInfo::from(ActivityType::OtherSports)),
            "rollerblading" => Ok(ActivityInfo::from(ActivityType::RollerBlading)),
            "running" => Ok(ActivityInfo::from(ActivityType::Running)),
            "skiing" => Ok(ActivityInfo::from(ActivityType::Skiing)),
            "soccer" => Ok(ActivityInfo::from(ActivityType::Soccer)),
            "softball" => Ok(ActivityInfo::from(ActivityType::Softball)),
            "swimming" => Ok(ActivityInfo::from(ActivityType::Swimming)),
            "tennis" => Ok(ActivityInfo::from(ActivityType::Tennis)),
            "track_and_field" => Ok(ActivityInfo::from(ActivityType::TrackAndField)),
            "volleyball" => Ok(ActivityInfo::from(ActivityType::VolleyBall)),
            "walking" => Ok(ActivityInfo::from(ActivityType::Walking)),
            _ => Err(()),
        }
    }
}

impl ActivityInfo {
    pub fn new(
        activity_type: ActivityType,
        available_data_points: ActivityDataPoints,
        average_calories_burned_per_minute: u32,
        id: &'static str,
        name: String,
    ) -> Self {
        Self {
            activity_type,
            available_data_points,
            average_calories_burned_per_minute,
            id,
            name,
        }
    }
}
