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
use num_traits::{FromPrimitive, ToPrimitive};
use uom::si::{
    f32::{Length, Mass},
    length::centimeter,
    mass::kilogram,
};

#[derive(Debug, Clone)]
pub struct Settings(gio::Settings);

static mut SETTINGS: Option<Settings> = None;

impl Settings {
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

    delegate::delegate! {
        to self.0 {
            pub fn bind<'a, P: IsA<glib::Object>>(
                &'a self,
                key: &'a str,
                object: &'a P,
                property: &'a str
            ) -> gio::BindingBuilder<'a>;

            pub fn connect_changed<F: Fn(&gio::Settings, &str) + 'static>(
                &self,
                detail: Option<&str>,
                f: F
            ) -> glib::SignalHandlerId;

            pub fn disconnect(&self, handler_id: glib::SignalHandlerId);

            fn enum_(&self, key: &str) -> i32;

            fn get<U: glib::FromVariant>(&self, key: &str) -> U;

            fn set<U: ToVariant>(&self, key: &str, value: &U) -> Result<(), glib::BoolError>;

            fn set_enum(&self, key: &str, value: i32) -> Result<(), glib::BoolError>;

            fn set_strv(&self, key: &str, value: &[&str]) -> Result<(), glib::BoolError>;

            fn set_string(&self, key: &str, value: &str) -> Result<(), glib::BoolError>;

            fn string(&self, key: &str) -> glib::GString;

            fn strv(&self, key: &str) -> Vec<glib::GString>;
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::instance()
    }
}

impl Settings {
    pub fn instance() -> Self {
        unsafe {
            SETTINGS.as_ref().map_or_else(
                || {
                    let settings = Self(gio::Settings::new("dev.Cogitri.Health"));
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
        Length::new::<centimeter>(self.get::<u32>("user-height") as f32)
    }

    /// Set the user's height.
    pub fn set_user_height(&self, value: Length) {
        self.set("user-height", &(value.get::<centimeter>() as u32))
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
        Mass::new::<kilogram>(self.get::<f64>("user-weightgoal") as f32)
    }

    /// Set the user's current weightgoal.
    pub fn set_user_weightgoal(&self, value: Mass) {
        self.set("user-weightgoal", &f64::from(value.get::<kilogram>()))
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::Settings;
    use crate::utils::init_gschema;
    use chrono::{DateTime, FixedOffset, Local};

    fn get() -> (Option<tempfile::TempDir>, Settings) {
        (init_gschema(), Settings::instance())
    }

    #[test]
    fn current_view_id() {
        let (_tmp, settings) = get();
        settings.set_current_view_id(settings.current_view_id());
    }

    #[test]
    fn did_initial_setup() {
        let (_tmp, settings) = get();
        settings.set_did_initial_setup(settings.did_initial_setup());
    }

    #[test]
    fn sync_provider_setup_google_fit() {
        let (_tmp, settings) = get();
        settings.set_sync_provider_setup_google_fit(settings.sync_provider_setup_google_fit());
    }

    #[test]
    fn user_age() {
        let (_tmp, settings) = get();
        settings.set_user_age(settings.user_age());
    }

    #[test]
    fn user_stepgoal() {
        let (_tmp, settings) = get();
        settings.set_user_stepgoal(settings.user_stepgoal());
    }

    #[test]
    fn window_height() {
        let (_tmp, settings) = get();
        settings.set_window_height(settings.window_height());
    }

    #[test]
    fn window_is_maximized() {
        let (_tmp, settings) = get();
        settings.set_window_is_maximized(settings.window_is_maximized());
    }

    #[test]
    fn window_width() {
        let (_tmp, settings) = get();
        settings.set_window_width(settings.window_width());
    }

    #[test]
    fn recent_activity_types() {
        let (_tmp, settings) = get();
        let types = settings.recent_activity_types();
        let s: Vec<&str> = types.iter().map(|s| &**s).collect();
        settings.set_recent_activity_types(&s);
    }

    #[test]
    fn timestamp_last_sync_google_fit() {
        let (_tmp, settings) = get();
        settings.set_timestamp_last_sync_google_fit(settings.timestamp_last_sync_google_fit());
    }

    #[test]
    fn unitsystem() {
        let (_tmp, settings) = get();
        settings.set_unitsystem(settings.unitsystem());
    }

    #[test]
    fn user_birthday() {
        let (_tmp, settings) = get();
        let d: DateTime<FixedOffset> = Local::now().into();
        settings.set_user_birthday(settings.user_birthday().unwrap_or(d.date()));
    }

    #[test]
    fn user_height() {
        let (_tmp, settings) = get();
        settings.set_user_height(settings.user_height());
    }

    #[test]
    fn user_weightgoal() {
        let (_tmp, settings) = get();
        settings.set_user_weightgoal(settings.user_weightgoal());
    }
}
