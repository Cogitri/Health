/* data_category.rs
 *
 * Copyright 2021-2022 Aman Kumar <amankrx.com>
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

/// All supported [DataCategory]s are listed in this enum.
#[derive(
    Debug,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum DataCategory {
    Activity,
    Weight,
}

impl Default for DataCategory {
    fn default() -> Self {
        Self::Activity
    }
}

impl glib::ToValue for DataCategory {
    fn to_value(&self) -> glib::Value {
        self.as_ref().to_value()
    }

    fn value_type(&self) -> glib::Type {
        <String as glib::StaticType>::static_type()
    }
}

#[cfg(test)]
mod test {
    use super::DataCategory;
    use std::str::FromStr;

    #[test]
    fn deserialize_data_category() {
        assert_eq!(
            DataCategory::from_str("activity"),
            Ok(DataCategory::Activity)
        );
        assert_eq!(DataCategory::from_str("weight"), Ok(DataCategory::Weight),);
        assert!(DataCategory::from_str("unknown").is_err());
    }

    #[test]
    fn serialize_data_category() {
        let a: &str = DataCategory::Activity.into();
        assert_eq!(a, "activity");
        let a: &str = DataCategory::Weight.into();
        assert_eq!(a, "weight");
    }
}
