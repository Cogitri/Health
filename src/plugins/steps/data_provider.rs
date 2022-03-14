/* data_provider.rs
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

use crate::{core::Database, model::Steps, prelude::*, views::Point};
use anyhow::Result;
use std::{collections::BTreeMap, convert::TryInto};

/// A [GraphModelSteps] manages step data for easy consumption in views.
#[derive(Debug, Default)]
pub struct GraphModelSteps {
    database: Database,
    vec: Vec<Steps>,
}

impl GraphModelSteps {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn new_with_database(database: Database) -> Self {
        Self {
            database,
            vec: Vec::new(),
        }
    }

    /// Get how many steps have been done today.
    ///
    /// # Returns
    /// The amount of steps that have been done today, none `None` if no steps have been done yet.
    pub fn today_step_count(&self) -> Option<u32> {
        let today = glib::DateTime::local();
        let today_steps = self
            .vec
            .iter()
            .filter_map(|s| {
                if today.reset_hms() == s.date.reset_hms() {
                    Some(s.steps)
                } else {
                    None
                }
            })
            .sum();
        if today_steps == 0 {
            None
        } else {
            Some(today_steps)
        }
    }

    /// Get how many days the user has upheld their step streak (as in have reached their step goal), including today.
    ///
    /// # Returns
    /// The number of days.
    pub fn streak_count_today(&self, step_goal: u32) -> u32 {
        let today = glib::DateTime::local();
        self.streak_count(step_goal, today)
    }

    /// Get how many days the user has upheld their step streak (as in have reached their step goal), excluding today.
    ///
    /// # Returns
    /// The number of days.
    pub fn streak_count_yesterday(&self, step_goal: u32) -> u32 {
        let yesterday = glib::DateTime::local().add_days(-1).unwrap();
        self.streak_count(step_goal, yesterday)
    }

    #[allow(clippy::map_entry)]
    fn streak_count(&self, step_goal: u32, date: glib::DateTime) -> u32 {
        let mut map = BTreeMap::new();
        for steps in &self.vec {
            let date = steps.date.reset_hms();
            if map.contains_key(&date) {
                map.insert(date.clone(), map.get(&date).unwrap() + steps.steps);
            } else {
                map.insert(date.clone(), steps.steps);
            }
        }

        let mut date_it = date.reset_hms().add_days(1).unwrap();
        map.into_iter()
            .rev()
            .skip_while(|(s_date, _)| *s_date > date.reset_hms()) // skip days which are newer than date - e.g. skip today if we want to get yesterday's streak count
            .take_while(|(s_date, steps)| {
                date_it = date_it.add_days(-1).unwrap();
                s_date.reset_hms() == date_it && *steps >= step_goal
            })
            .count()
            .try_into()
            .unwrap()
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
            .steps(glib::DateTime::local().subtract(duration))
            .await?;
        Ok(())
    }

    /// Converts the model's data to an array of `Point` so it can be displayed in a `GraphView`.
    #[allow(clippy::map_entry)]
    pub fn to_points(&self) -> Vec<crate::views::Point> {
        if self.vec.is_empty() {
            return Vec::new();
        }

        let first_date = self.vec.first().unwrap().date.reset_hms();
        let mut map = BTreeMap::new();

        for steps in &self.vec {
            let date = steps.date.date();
            if map.contains_key(&date) {
                map.insert(date, map.get(&date).unwrap() + steps.steps);
            } else {
                map.insert(date, steps.steps);
            }
        }

        for date_delta in 0..(first_date.difference(&glib::DateTime::local()).as_days()) {
            map.entry(
                first_date
                    .add_days(date_delta.try_into().unwrap())
                    .unwrap()
                    .date(),
            )
            .or_insert(0);
        }

        map.into_iter()
            .map(|(date, steps)| Point {
                date: date.and_time_local(Time::new(12, 0, 0).unwrap()),
                value: steps as f32,
            })
            .collect::<Vec<Point>>()
    }

    /// Get if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::GraphModelSteps;
    use crate::{core::Database, model::Activity, prelude::*, views::Point};
    use gtk::glib;
    use tempfile::tempdir;

    #[test]
    fn streak_count() {
        let ctx = glib::MainContext::new();
        ctx.with_thread_default(|| {
            let data_dir = tempdir().unwrap();
            let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
            let now = glib::DateTime::local();

            let mut model = GraphModelSteps::new_with_database(db.clone());
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.streak_count_today(5000), 0);

            let act = Activity::new();
            act.set_steps(Some(5000));
            act.set_date(now.clone());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.streak_count_today(5000), 1);

            let act = Activity::new();
            act.set_steps(Some(8000));
            act.set_date(now.clone().add_days(-1).unwrap().add_hours(-1).unwrap());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(30)))
                .unwrap();
            assert_eq!(model.streak_count_today(5000), 2);
            assert_eq!(model.streak_count_today(8000), 0);
            assert_eq!(model.streak_count_yesterday(5000), 1);
            assert_eq!(model.streak_count_yesterday(8000), 1);

            let act = Activity::new();
            act.set_steps(Some(400));
            act.set_date(now.add_days(-1).unwrap());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(30)))
                .unwrap();
            assert_eq!(model.streak_count_today(5000), 2);
        })
        .unwrap();
    }

    #[test]
    fn to_points() {
        let ctx = glib::MainContext::new();
        ctx.with_thread_default(|| {
            let data_dir = tempdir().unwrap();
            let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

            let mut model = GraphModelSteps::new_with_database(db.clone());
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.to_points(), vec![]);

            let act = Activity::new();
            act.set_steps(Some(5000));
            let date = glib::DateTime::local();
            act.set_date(date.clone());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![Point {
                    date: date.clone(),
                    value: 5000.0
                }]
            );

            let act = Activity::new();
            act.set_steps(Some(8000));
            let date_y = date.clone().add_days(-1).unwrap().add_hours(-1).unwrap();
            act.set_date(date_y.clone());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(30)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![
                    Point {
                        date: date_y.clone(),
                        value: 8000.0
                    },
                    Point {
                        date: date.clone(),
                        value: 5000.0
                    }
                ]
            );

            let act = Activity::new();
            act.set_steps(Some(400));
            act.set_date(date.add_days(-1).unwrap());
            ctx.block_on(db.save_activity(act)).unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(30)))
                .unwrap();
            assert_eq!(
                model.to_points(),
                vec![
                    Point {
                        date: date_y,
                        value: 8400.0
                    },
                    Point {
                        date: date,
                        value: 5000.0
                    }
                ]
            );
        })
        .unwrap();
    }

    #[test]
    fn today_step_count() {
        let ctx = glib::MainContext::new();
        ctx.with_thread_default(|| {
            let data_dir = tempdir().unwrap();
            let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
            let now = glib::DateTime::local();
            ctx.block_on(
                db.save_activity(Activity::builder().date(now.clone()).steps(2000).build()),
            )
            .unwrap();
            let mut model = GraphModelSteps::new_with_database(db.clone());
            assert!(model.today_step_count().is_none());
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.today_step_count(), Some(2000));
            ctx.block_on(db.save_activity(Activity::builder().date(now).steps(4000).build()))
                .unwrap();
            ctx.block_on(model.reload(glib::TimeSpan::from_days(1)))
                .unwrap();
            assert_eq!(model.today_step_count(), Some(6000));
        })
        .unwrap();
    }
}
