use crate::{core::Database, model::Steps, views::Point};
use chrono::{Duration, Local};
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct GraphModelSteps {
    database: Database,
    vec: Vec<Steps>,
}

impl GraphModelSteps {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            vec: Vec::new(),
        }
    }

    pub fn get_today_step_count(&self) -> Option<u32> {
        let today = chrono::Local::now().date();
        self.vec
            .iter()
            .find(|s| today == s.date.date())
            .map(|s| s.steps)
    }

    pub fn get_streak_count_today(&self, step_goal: u32) -> u32 {
        let vec: Vec<&Steps> = self.vec.iter().collect();
        GraphModelSteps::get_streak_count(&vec, step_goal)
    }

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
        let last_date = steps.get(0).unwrap().date;

        for x in steps.iter() {
            if u32::try_from(last_date.signed_duration_since(x.date).num_days()).unwrap() == streak
                && x.steps >= step_goal
            {
                streak += 1;
            } else {
                break;
            }
        }

        streak
    }

    pub async fn reload(&mut self, duration: Duration) -> Result<(), glib::Error> {
        self.vec = self
            .database
            .get_steps(
                chrono::Local::now()
                    .checked_sub_signed(duration)
                    .unwrap()
                    .into(),
            )
            .await?;
        Ok(())
    }

    pub fn to_points(&self) -> Vec<crate::views::Point> {
        if self.vec.is_empty() {
            return Vec::new();
        }

        let first_date = self.vec.first().unwrap().date;
        let mut last_val = 0;
        let mut ret = Vec::with_capacity(self.vec.len());

        for (i, point) in self.vec.iter().enumerate() {
            for j in i..last_val {
                let date = first_date
                    .clone()
                    .checked_add_signed(Duration::days((i + j).try_into().unwrap()))
                    .unwrap();
                ret.push(Point { date, value: 0.0 });
            }
            ret.push(Point {
                date: point.date,
                value: point.steps as f32,
            });
            last_val = point
                .date
                .signed_duration_since(first_date)
                .num_days()
                .try_into()
                .unwrap();
        }

        for x in last_val
            ..usize::try_from(Local::now().signed_duration_since(first_date).num_days()).unwrap()
        {
            let date = first_date
                .clone()
                .checked_add_signed(Duration::days(x.try_into().unwrap()))
                .unwrap();
            ret.push(Point { date, value: 0.0 });
        }

        if ret.last().unwrap().date.date() != Local::now().date() {
            ret.push(Point {
                date: Local::now().into(),
                value: 0.0,
            });
        }

        ret
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
