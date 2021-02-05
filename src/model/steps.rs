/* steps.rs
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

use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct Steps {
    pub date: DateTime<FixedOffset>,
    pub steps: u32,
}

impl Steps {
    pub fn new(date: DateTime<FixedOffset>, steps: u32) -> Self {
        Self { date, steps }
    }
}
