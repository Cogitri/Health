/* activity_data_points.rs
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

bitflags::bitflags! {
    /// An [ActivityDataPoints] are different data points that an [Activity](crate::model::Activity) can hold.
    pub struct ActivityDataPoints: u16 {
        const CALORIES_BURNED = 0b00_0001;
        const DISTANCE = 0b00_0010;
        const DURATION = 0b00_0100;
        const HEART_RATE = 0b00_1000;
        const STEP_COUNT = 0b01_0000;
    }
}
