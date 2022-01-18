/* notification_frequency.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
 * Copyright 2021-2022 Rasmus Thomsen <oss@cogitri.dev>
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

use gtk::glib;

#[derive(
    PartialEq,
    Debug,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    strum::EnumString,
    strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum NotificationFrequency {
    Hourly,
    Every4Hrs,
    Fixed,
}

impl Default for NotificationFrequency {
    fn default() -> Self {
        Self::Every4Hrs
    }
}

impl glib::ToValue for NotificationFrequency {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as glib::StaticType>::static_type()
    }
}
