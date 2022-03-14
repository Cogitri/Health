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

use crate::{prelude::*, views::Point};
use anyhow::Result;
use uom::si::{f32::Mass, mass::kilogram};

#[derive(Debug, Default, Clone)]
pub struct GraphModelWeightMocked {}

impl GraphModelWeightMocked {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn reload(&mut self, _duration: glib::TimeSpan) -> Result<()> {
        Ok(())
    }

    pub fn to_points(&self) -> Vec<Point> {
        let now = glib::DateTime::local();
        vec![
            Point {
                date: now.add_days(-60).unwrap(),
                value: 65.0,
            },
            Point {
                date: now.add_days(-30).unwrap(),
                value: 64.2,
            },
            Point {
                date: now,
                value: 63.6,
            },
        ]
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn last_weight(&self) -> Option<Mass> {
        Some(Mass::new::<kilogram>(63.6))
    }
}
