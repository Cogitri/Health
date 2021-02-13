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

    /// Reload the data from the Tracker Database.
    ///
    /// # Arguments
    /// * `duration` - How far in the past the data should reach back.
    ///
    /// # Returns
    /// Returns an error if querying the DB fails.
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

    /// Converts the model's data to an array of [Point](crate::views::Point) so it can be displayed in a [GraphView](crate::views::GraphView).
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

    /// Get if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Get the last weight the user added.
    pub fn get_last_weight(&self) -> Option<Mass> {
        self.vec.last().map(|w| w.weight)
    }
}
