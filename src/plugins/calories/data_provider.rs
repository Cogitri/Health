/* graph_model_calories.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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
    core::{Database, Settings},
    model::ActivityType,
    prelude::*,
    views::SplitBar,
};
use anyhow::Result;
use std::collections::HashMap;
use uom::si::{f32::Mass, length::centimeter, mass::kilogram};

/// A [GraphModelCalories] manages calories data for easy consumption in views.
#[derive(Debug, Default)]
pub struct GraphModelCalories {
    database: Database,
    settings: Settings,
    pub distinct_activities: Vec<ActivityType>,
    pub rmr: f32,
    split_bar: Vec<SplitBar>,
}

impl GraphModelCalories {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn new_with_database(database: Database, settings: Settings) -> Self {
        Self {
            database,
            settings,
            distinct_activities: Vec::new(),
            split_bar: Vec::new(),
            rmr: 0.0,
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
        let date = glib::DateTime::local().subtract(duration);
        self.distinct_activities = self.database.most_frequent_activities(date.clone()).await?;
        self.split_bar = self.database.calories(date).await?;
        self.rmr = self.rmr().await;
        Ok(())
    }

    /// Converts the model's data to an array of `SplitBars` so it can be displayed in a `BarGraphView`.
    pub fn to_split_bar(&self) -> Vec<crate::views::SplitBar> {
        if self.split_bar.is_empty() {
            return Vec::new();
        }

        let first_date = self.split_bar.first().unwrap().date.clone();
        let mut map = HashMap::new();

        for bar in &self.split_bar {
            map.insert(bar.date.clone(), bar.calorie_split.clone());
        }

        for date_delta in 0..first_date.difference(&glib::DateTime::local()).as_days() {
            map.entry(first_date.add_days(date_delta.try_into().unwrap()).unwrap())
                .or_insert_with(HashMap::new);
        }

        map.into_iter()
            .map(|(date, split_bar)| SplitBar {
                date,
                calorie_split: split_bar,
            })
            .collect::<Vec<SplitBar>>()
    }

    pub fn distinct_activities(&self) -> &[ActivityType] {
        &self.distinct_activities
    }

    pub async fn rmr(&self) -> f32 {
        let weights = match Database::instance().weights(None).await {
            Err(e) => {
                glib::g_warning!(crate::config::LOG_DOMAIN, "Failed to load weight data: {e}",);
                return 0.0;
            }
            Ok(v) => v,
        };
        let user_id = i64::from(self.settings.active_user_id());
        let user = &self.database.user(user_id).await.unwrap();
        let weight = weights
            .last()
            .map_or_else(|| Mass::new::<kilogram>(0.0), |w| w.weight)
            .get::<kilogram>();
        let height = user.user_height().unwrap().get::<centimeter>();
        let age = user
            .user_birthday()
            .unwrap()
            .difference(&glib::DateTime::local())
            .as_years() as f32;
        if weight != 0.0 {
            9.99 * weight + 6.25 * height - 4.92 * age
        } else {
            0.0
        }
    }

    /// Get if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.split_bar.is_empty()
    }
}
