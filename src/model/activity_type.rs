use crate::core::i18n_f;
use std::convert::TryFrom;

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive, Clone)]
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
