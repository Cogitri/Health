/* graph_model_steps.rs
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

use crate::{core::Database, model::Steps, views::Point};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use std::convert::{TryFrom, TryInto};

/// A [GraphModelSteps] manages step data for easy consumption in views.
#[derive(Debug)]
pub struct GraphModelSteps {
    database: Database,
    vec: Vec<Steps>,
}

/// Clone implementation where we don't clone the vec since that'd be expensive. We only
/// clone in view_weight to avoid holding a `RefCell`s  `Ref` for too long.
impl Clone for GraphModelSteps {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            vec: Vec::new(),
        }
    }
}

impl GraphModelSteps {
    pub fn new() -> Self {
        Self {
            database: Database::get_instance(),
            vec: Vec::new(),
        }
    }

    /// Get how many steps have been done today.
    ///
    /// # Returns
    /// The amount of steps that have been done today, none `None` if no steps have been done yet.
    pub fn get_today_step_count(&self) -> Option<u32> {
        let today = chrono::Local::now().date();
        self.vec.iter().find_map(|s| {
            if today == s.date.date() {
                Some(s.steps)
            } else {
                None
            }
        })
    }

    /// Get how many days the user has upheld their step streak (as in have reached their stepgoal), including today.
    ///
    /// # Returns
    /// The number of days.
    pub fn get_streak_count_today(&self, step_goal: u32) -> u32 {
        let vec: Vec<&Steps> = self.vec.iter().collect();
        GraphModelSteps::get_streak_count(&vec, step_goal)
    }

    /// Get how many days the user has upheld their step streak (as in have reached their stepgoal), excluding today.
    ///
    /// # Returns
    /// The number of days.
    pub fn get_streak_count_yesterday(&self, step_goal: u32) -> u32 {
        let today = chrono::Local::now().date();
        let vec: Vec<&Steps> = self.vec.iter().filter(|s| s.date.date() != today).collect();

        GraphModelSteps::get_streak_count(&vec, step_goal)
    }

    fn get_streak_count(steps: &[&Steps], step_goal: u32) -> u32 {
        if steps.is_empty() {
            return 0;
        }

        let mut streak: u32 = 0;
        let earliest_date = steps.get(0).unwrap().date;

        for x in steps.iter() {
            if u32::try_from((x.date - earliest_date).num_days()).unwrap() == streak
                && x.steps >= step_goal
            {
                streak += 1;
            } else {
                break;
            }
        }

        streak
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
            .get_steps((chrono::Local::now() - duration).into())
            .await?;
        Ok(())
    }

    /// Converts the model's data to an array of `Point` so it can be displayed in a `GraphView`.
    pub fn to_points(&self) -> Vec<crate::views::Point> {
        if self.vec.is_empty() {
            return Vec::new();
        }

        let first_date = self.vec.first().unwrap().date.date();
        let mut last_val = 0;
        let mut ret = Vec::with_capacity(self.vec.len());

        for (i, point) in self.vec.iter().enumerate() {
            for j in i..last_val {
                let date = first_date + Duration::days((i + j).try_into().unwrap());
                ret.push(Point { date, value: 0.0 });
            }
            ret.push(Point {
                date: point.date.date(),
                value: point.steps as f32,
            });

            last_val = point
                .date
                .date()
                .signed_duration_since(first_date)
                .num_days()
                .try_into()
                .unwrap();
        }

        for x in last_val
            ..usize::try_from(
                Utc::now()
                    .date()
                    .signed_duration_since(first_date)
                    .num_days(),
            )
            .unwrap()
        {
            let date = first_date + Duration::days(x.try_into().unwrap());
            ret.push(Point { date, value: 0.0 });
        }

        if ret.last().unwrap().date != Utc::now().date() {
            ret.push(Point {
                date: DateTime::<FixedOffset>::from(Utc::now()).date(),
                value: 0.0,
            });
        }

        ret
    }

    /// Get if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
