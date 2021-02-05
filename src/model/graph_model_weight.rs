use crate::{
    core::{settings::Unitsystem, Database, Settings},
    model::weight::Weight,
    views::Point,
};
use chrono::Duration;
use uom::si::{
    f32::Mass,
    mass::{kilogram, pound},
};

#[derive(Debug)]
pub struct GraphModelWeight {
    database: Database,
    settings: Settings,
    vec: Vec<Weight>,
}

/// Clone implementation where we don't clone the vec since that'd be expensive. We only
/// clone in view_weight to avoid holding a `RefCell`s  `Ref` for too long.
impl Clone for GraphModelWeight {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            settings: self.settings.clone(),
            vec: Vec::new(),
        }
    }
}

impl GraphModelWeight {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            settings: Settings::new(),
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
                    date: w.date.date(),
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
