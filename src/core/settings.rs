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
use chrono::{Date, DateTime, FixedOffset};
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
    pub fn instance() -> Self {
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
    pub fn recent_activity_types(&self) -> Vec<String> {
        self.strv("recent-activity-types")
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Set an array of recent activity IDs.
    pub fn set_recent_activity_types(&self, value: &[&str]) {
        self.set_strv("recent-activity-types", value).unwrap();
    }

    /// Get the timestamp of the last sync with Google Fit.
    pub fn timestamp_last_sync_google_fit(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(self.string("timestamp-last-sync-google-fit").as_str())
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
    pub fn unitsystem(&self) -> Unitsystem {
        Unitsystem::from_i32(self.enum_("unitsystem")).unwrap()
    }

    /// Set the current unitsystem.
    pub fn set_unitsystem(&self, value: Unitsystem) {
        self.set_enum("unitsystem", value.to_i32().unwrap())
            .unwrap();
    }

    /// Get the timestamp of the last sync with Google Fit.
    pub fn user_birthday(&self) -> Option<Date<FixedOffset>> {
        let str = self.string("user-birthday");

        if str.is_empty() {
            None
        } else {
            Some(
                DateTime::parse_from_rfc3339(str.as_str())
                    .map(|s| s.date())
                    .unwrap(),
            )
        }
    }

    /// Set the timestamp of the last sync with Google Fit.
    pub fn set_user_birthday(&self, value: Date<FixedOffset>) {
        self.set_string("user-birthday", &value.and_hms(0, 0, 0).to_rfc3339())
            .unwrap();
    }

    /// Get the user's height.
    pub fn user_height(&self) -> Length {
        Length::new::<centimeter>(self.uint("user-height") as f32)
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
    pub fn user_weightgoal(&self) -> Mass {
        Mass::new::<kilogram>(self.double("user-weightgoal") as f32)
    }

    /// Set the user's current weightgoal.
    pub fn set_user_weightgoal(&self, value: Mass) {
        self.set_double("user-weightgoal", f64::from(value.get::<kilogram>()))
            .unwrap();
    }
}
