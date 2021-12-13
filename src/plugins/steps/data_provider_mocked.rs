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

use crate::views::Point;
use anyhow::Result;
use chrono::{DateTime, Duration, FixedOffset, Local};

#[derive(Debug, Default, Clone)]
pub struct GraphModelStepsMocked {}

impl GraphModelStepsMocked {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn today_step_count(&self) -> Option<u32> {
        Some(8500)
    }

    pub fn streak_count_today(&self, _step_goal: u32) -> u32 {
        3
    }

    pub fn streak_count_yesterday(&self, _step_goal: u32) -> u32 {
        2
    }

    pub async fn reload(&mut self, _duration: Duration) -> Result<()> {
        Ok(())
    }

    pub fn to_points(&self) -> Vec<Point> {
        let now: DateTime<FixedOffset> = Local::now().into();
        vec![
            Point {
                date: now.date() - Duration::days(2),
                value: 10200.0,
            },
            Point {
                date: now.date() - Duration::days(1),
                value: 9700.0,
            },
            Point {
                date: now.date(),
                value: 8500.0,
            },
        ]
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}
