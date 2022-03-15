/* graph_model_weight.rs
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
    core::{Database, Settings, UnitSystem},
    model::weight::Weight,
    prelude::*,
    views::Point,
};
use anyhow::Result;
use std::collections::BTreeMap;
use uom::si::{
    f32::Mass,
    mass::{kilogram, pound},
};

/// A [GraphModelWeight] manages weight data for easy consumption in views.
#[derive(Debug, Default)]
pub struct GraphModelWeight {
    database: Database,
    settings: Settings,
    vec: Vec<Weight>,
}

impl GraphModelWeight {
    pub fn new() -> Self {
        Self {
            database: Database::instance(),
            settings: Settings::instance(),
            vec: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn new_with_database(database: Database) -> Self {
        Self {
            database,
            settings: Settings::instance(),
            vec: Vec::new(),
        }
    }

    /// Reload the data from the Tracker Database.
    ///
    /// # Arguments
    /// * `duration` - How far in the past the data should reach back.
    ///
    /// # Returns
    /// Returns an error if querying the DB fails.
    pub async fn reload(&mut self, duration: glib::TimeSpan) -> Result<()> {
        self.vec = self
            .database
            .weights(Some(glib::DateTime::local().subtract(duration)))
            .await?;
        Ok(())
    }

    /// Converts the model's data to an array of [Point](crate::views::Point) so it can be displayed in a [GraphView](crate::views::GraphView).
    pub fn to_points(&self) -> Vec<crate::views::Point> {
        let map = self
            .vec
            .iter()
            .map(|w| {
                let val = if self.settings.unit_system() == UnitSystem::Metric {
                    w.weight.get::<kilogram>()
                } else {
                    w.weight.get::<pound>()
                };

                (w.date.clone(), val)
            })
            .collect::<BTreeMap<_, _>>();

        map.into_iter()
            .map(|(date, value)| Point { date, value })
            .collect()
    }

    /// Get if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Get the last weight the user added.
    pub fn last_weight(&self) -> Option<Mass> {
        self.vec.last().map(|w| w.weight)
    }

    pub fn penultimate_weight(&self) -> Option<Mass> {
        if self.vec.len() == 1 {
            self.last_weight()
        } else {
            Some(self.vec[self.vec.len() - 2].weight)
        }
    }
}

#[cfg(test)]
mod test {
    use super::GraphModelWeight;
    use crate::{core::Database, model::Weight, prelude::*, views::Point};
    use gtk::glib;
    use tempfile::tempdir;
    use uom::si::{f32::Mass, mass::kilogram};

    #[test]
    fn to_points() {
        let ctx = glib::MainContext::new();
        ctx.with_thread_default(|| {
            let data_dir = tempdir().unwrap();
            let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

            let mut model = GraphModelWeight::new_with_database(db.clone());
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.to_points(), vec![]);

            let date = glib::DateTime::local();
            let weight = Weight::new(date.clone(), Mass::new::<kilogram>(42.0));
            ctx.block_on(db.save_weight(weight)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![Point {
                    date: date.clone(),
                    value: 42.0,
                }]
            );

            let weight = Weight::new(
                date.clone().add_days(-1).unwrap(),
                Mass::new::<kilogram>(43.0),
            );
            ctx.block_on(db.save_weight(weight)).unwrap();
            // 5 second buffer since it might take a second or two to save the weight
            ctx.block_on(model.reload(glib::TimeSpan::from_seconds(3600 * 24 + 5)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![
                    Point {
                        date: date.clone().add_days(-1).unwrap(),
                        value: 43.0,
                    },
                    Point {
                        date: date.clone(),
                        value: 42.0,
                    }
                ]
            );

            let weight = Weight::new(date.clone(), Mass::new::<kilogram>(43.0));
            ctx.block_on(db.save_weight(weight)).unwrap();
            // 5 second buffer since it might take a second or two to save the weight
            ctx.block_on(model.reload(glib::TimeSpan::from_seconds(3600 * 24 + 5)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![
                    Point {
                        date: date.clone().add_days(-1).unwrap(),
                        value: 43.0,
                    },
                    Point { date, value: 43.0 }
                ]
            );
        })
        .unwrap();
    }
}
