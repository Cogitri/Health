use crate::settings_getter_setter;
use chrono::{DateTime, FixedOffset};
use gio::prelude::*;
use gio::Settings;
use num_traits::{FromPrimitive, ToPrimitive};
use uom::si::{
    f32::{Length, Mass},
    length::centimeter,
    mass::kilogram,
};

#[derive(PartialEq, Debug, Clone, Copy, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum Unitsystem {
    Imperial,
    Metric,
}

#[derive(Debug, Clone)]
pub struct HealthSettings {
    settings: Settings,
}

impl HealthSettings {
    pub fn new() -> Self {
        Self {
            settings: Settings::new("dev.Cogitri.Health"),
        }
    }

    pub fn get_recent_activity_types(&self) -> Vec<String> {
        self.settings
            .get_strv("recent-activity-types")
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn set_recent_activity_types(&self, value: &[&str]) {
        self.settings
            .set_strv("recent-activity-types", value)
            .unwrap();
    }

    pub fn get_timestamp_last_sync_google_fit(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(
            self.settings
                .get_string("timestamp-last-sync-google-fit")
                .unwrap()
                .as_str(),
        )
        .unwrap()
    }

    pub fn set_timestamp_last_sync_google_fit(&self, value: DateTime<FixedOffset>) {
        self.settings
            .set_string("timestamp-last-sync-google-fit", &value.to_rfc3339())
            .unwrap();
    }

    pub fn connect_unitsystem_changed<F: Fn(&Settings, &str) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.settings.connect_changed(move |s, name| {
            if name == "unitsystem" {
                f(s, name);
            }
        })
    }

    pub fn get_unitsystem(&self) -> Unitsystem {
        Unitsystem::from_i32(self.settings.get_enum("unitsystem")).unwrap()
    }

    pub fn set_unitsystem(&self, value: Unitsystem) {
        self.settings
            .set_enum("unitsystem", value.to_i32().unwrap())
            .unwrap();
    }

    pub fn get_user_height(&self) -> Length {
        Length::new::<centimeter>(self.settings.get_uint("user-height") as f32)
    }

    pub fn set_user_height(&self, value: Length) {
        self.settings
            .set_uint("user-height", value.get::<centimeter>() as u32)
            .unwrap();
    }

    pub fn connect_user_weightgoal_changed<F: Fn(&Settings, &str) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.settings.connect_changed(move |s, name| {
            if name == "user-weightgoal" {
                f(s, name);
            }
        })
    }

    pub fn get_user_weightgoal(&self) -> Mass {
        Mass::new::<kilogram>(self.settings.get_double("user-weightgoal") as f32)
    }

    pub fn set_user_weightgoal(&self, value: Mass) {
        self.settings
            .set_double("user-weightgoal", value.get::<kilogram>() as f64)
            .unwrap();
    }

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
}
