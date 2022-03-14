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

use crate::{model::ActivityType, prelude::*, views::SplitBar};
use anyhow::Result;
use gtk::glib;
use std::collections::HashMap;

/// A [GraphModelCaloriesMocked] manages calories data for easy consumption in views.
#[derive(Debug, Default)]
pub struct GraphModelCaloriesMocked {
    pub distinct_activities: Vec<ActivityType>,
    pub rmr: f32,
}

impl GraphModelCaloriesMocked {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn reload(&mut self, _duration: glib::TimeSpan) -> Result<()> {
        Ok(())
    }

    /// Converts the model's data to an array of `SplitBars` so it can be displayed in a `BarGraphView`.
    pub fn to_split_bar(&self) -> Vec<SplitBar> {
        let now = glib::DateTime::local();
        vec![
            SplitBar {
                date: now.clone(),
                calorie_split: HashMap::from([
                    (ActivityType::Basketball, 50),
                    (ActivityType::Walking, 150),
                ]),
            },
            SplitBar {
                date: now.add_days(-1).unwrap(),
                calorie_split: HashMap::from([
                    (ActivityType::Swimming, 250),
                    (ActivityType::Walking, 150),
                ]),
            },
            SplitBar {
                date: now.add_days(-2).unwrap(),
                calorie_split: HashMap::from([
                    (ActivityType::Basketball, 220),
                    (ActivityType::Running, 380),
                ]),
            },
        ]
    }

    pub fn distinct_activities(&self) -> &[ActivityType] {
        &[
            ActivityType::Walking,
            ActivityType::Basketball,
            ActivityType::Swimming,
            ActivityType::Running,
        ]
    }

    pub async fn rmr(&self) -> f32 {
        1500.0
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}
