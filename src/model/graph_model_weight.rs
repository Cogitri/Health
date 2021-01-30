use crate::{
    core::{settings::Unitsystem, HealthDatabase, HealthSettings},
    model::weight::*,
    views::Point,
};
use chrono::Duration;
use uom::si::{
    f32::Mass,
    mass::{kilogram, pound},
};

#[derive(Debug)]
pub struct HealthGraphModelWeight {
    database: HealthDatabase,
    settings: HealthSettings,
    vec: Vec<Weight>,
}

impl HealthGraphModelWeight {
    pub fn new(database: HealthDatabase) -> Self {
        Self {
            database,
            settings: HealthSettings::new(),
            vec: Vec::new(),
        }
    }

    pub async fn reload(&mut self, duration: Duration) -> Result<(), glib::Error> {
        self.vec = self
            .database
            .get_weights(Some(
                chrono::Local::now()
                    .checked_sub_signed(duration)
                    .unwrap()
                    .into(),
            ))
            .await?;
        Ok(())
    }

    pub fn to_points(&self) -> Vec<crate::views::Point> {
        self.vec
            .iter()
            .map(|w| {
                let val = if self.settings.get_unitsystem() == Unitsystem::Metric {
                    w.weight.get::<kilogram>()
                } else {
                    w.weight.get::<pound>()
                };

                Point {
                    date: w.date,
                    value: val,
                }
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn get_last_weight(&self) -> Option<Mass> {
        self.vec.last().map(|w| w.weight)
    }
}
