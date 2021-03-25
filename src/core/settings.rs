/* settings.rs
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

use crate::{core::Unitsystem, settings_getter_setter};
use chrono::{DateTime, FixedOffset};
use gio::prelude::*;
use gio::Settings;
use num_traits::{FromPrimitive, ToPrimitive};
use uom::si::{
    f32::{Length, Mass},
    length::centimeter,
    mass::kilogram,
};

pub mod prelude {
    pub use super::*;
}

static mut SETTINGS: Option<Settings> = None;

settings_getter_setter!(u32, current_view_id, "current-view-id");
settings_getter_setter!(bool, did_initial_setup, "did-initial-setup");
settings_getter_setter!(
    bool,
    sync_provider_setup_google_fit,
    "sync-provider-setup-google-fit"
);
settings_getter_setter!(u32, user_age, "user-age");
settings_getter_setter!(u32, user_stepgoal, "user-stepgoal");
settings_getter_setter!(i32, window_height, "window-height");
settings_getter_setter!(bool, window_is_maximized, "window-is-maximized");
settings_getter_setter!(i32, window_width, "window-width");

#[easy_ext::ext(HealthSettingsExt)]
impl Settings {
    pub fn get_instance() -> Self {
        unsafe {
            SETTINGS.as_ref().map_or_else(
                || {
                    let settings = Settings::new("dev.Cogitri.Health");
                    SETTINGS = Some(settings.clone());
                    settings
                },
                std::clone::Clone::clone,
            )
        }
    }

    /// Get an array of recent activity IDs.
    pub fn get_recent_activity_types(&self) -> Vec<String> {
        self.get_strv("recent-activity-types")
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Set an array of recent activity IDs.
    pub fn set_recent_activity_types(&self, value: &[&str]) {
        self.set_strv("recent-activity-types", value).unwrap();
    }

    /// Get the timestamp of the last sync with Google Fit.
    pub fn get_timestamp_last_sync_google_fit(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(
            self.get_string("timestamp-last-sync-google-fit")
                .unwrap()
                .as_str(),
        )
        .unwrap()
    }

    /// Set the timestamp of the last sync with Google Fit.
    pub fn set_timestamp_last_sync_google_fit(&self, value: DateTime<FixedOffset>) {
        self.set_string("timestamp-last-sync-google-fit", &value.to_rfc3339())
            .unwrap();
    }

    /// Connect to the `unitsystem` key changing. Keep in mind that the key has to be read once before connecting or this won't do anything!
    pub fn connect_unitsystem_changed<F: Fn(&gio::Settings, &str) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_changed(Some("unitsystem"), move |s, name| {
            f(s, name);
        })
    }

    /// Get the current unitsystem.
    pub fn get_unitsystem(&self) -> Unitsystem {
        Unitsystem::from_i32(self.get_enum("unitsystem")).unwrap()
    }

    /// Set the current unitsystem.
    pub fn set_unitsystem(&self, value: Unitsystem) {
        self.set_enum("unitsystem", value.to_i32().unwrap())
            .unwrap();
    }

    /// Get the user's height.
    pub fn get_user_height(&self) -> Length {
        Length::new::<centimeter>(self.get_uint("user-height") as f32)
    }

    /// Set the user's height.
    pub fn set_user_height(&self, value: Length) {
        self.set_uint("user-height", value.get::<centimeter>() as u32)
            .unwrap();
    }

    /// Connect to the `user-weightgoal` key changing. Keep in mind that the key has to be read once before connecting or this won't do anything!
    pub fn connect_user_weightgoal_changed<F: Fn(&gio::Settings, &str) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_changed(Some("user-weightgoal"), move |s, name| {
            f(s, name);
        })
    }

    /// Get the user's current weightgoal.
    pub fn get_user_weightgoal(&self) -> Mass {
        Mass::new::<kilogram>(self.get_double("user-weightgoal") as f32)
    }

    /// Set the user's current weightgoal.
    pub fn set_user_weightgoal(&self, value: Mass) {
        self.set_double("user-weightgoal", f64::from(value.get::<kilogram>()))
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::Unitsystem;
    use std::str::FromStr;

    #[test]
    fn deserialize_unitsystem() {
        assert_eq!(Unitsystem::from_str("imperial"), Ok(Unitsystem::Imperial));
        assert_eq!(Unitsystem::from_str("metric"), Ok(Unitsystem::Metric));

        assert!(Unitsystem::from_str("unknown").is_err());
    }

    #[test]
    fn serialize_unitsystem() {
        let a: &str = Unitsystem::Imperial.into();
        assert_eq!(a, "imperial");
        let b: &str = Unitsystem::Metric.into();
        assert_eq!(b, "metric");
    }
}
