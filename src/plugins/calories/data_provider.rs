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
    views::SplitBar,
};
use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, FixedOffset, Utc};
use gtk::glib;
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

/// Clone implementation where we don't clone the split_bar vector since that'd be expensive. We only
/// clone in view_weight to avoid holding a `RefCell`s  `Ref` for too long.
impl Clone for GraphModelCalories {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            settings: self.settings.clone(),
            distinct_activities: Vec::new(),
            rmr: 0.0,
            split_bar: Vec::new(),
        }
    }
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
    pub async fn reload(&mut self, duration: Duration) -> Result<()> {
        self.distinct_activities = self
            .database
            .most_frequent_activities((chrono::Local::now() - duration).into())
            .await?;
        self.split_bar = self
            .database
            .calories((chrono::Local::now() - duration).into())
            .await?;
        self.rmr = self.rmr().await;
        Ok(())
    }

    /// Converts the model's data to an array of `SplitBars` so it can be displayed in a `BarGraphView`.
    pub fn to_split_bar(&self) -> Vec<crate::views::SplitBar> {
        if self.split_bar.is_empty() {
            return Vec::new();
        }

        let first_date = self.split_bar.first().unwrap().date;
        let mut map = HashMap::new();

        for bar in &self.split_bar {
            map.insert(bar.date, bar.calorie_split.clone());
        }

        for date_delta in
            0..(DateTime::<FixedOffset>::from(Utc::now()).date() - first_date).num_days()
        {
            map.entry(first_date + Duration::days(date_delta))
                .or_insert_with(HashMap::new);
        }

        map.into_iter()
            .map(|(date, split_bar)| SplitBar {
                date,
                calorie_split: split_bar,
            })
            .collect::<Vec<SplitBar>>()
    }

    pub async fn rmr(&self) -> f32 {
        let weights = match Database::instance().weights(None).await {
            Err(e) => {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to load weight data: {}",
                    e
                );
                return 0.0;
            }
            Ok(v) => v,
        };
        let weight = weights
            .last()
            .map_or_else(|| Mass::new::<kilogram>(0.0), |w| w.weight)
            .get::<kilogram>() as f32;
        let height = self.settings.user_height().get::<centimeter>() as f32;
        let age =
            (chrono::Local::now().year() - self.settings.user_birthday().unwrap().year()) as f32;
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
