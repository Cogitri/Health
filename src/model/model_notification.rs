/* model_notification.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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

use crate::{i18n, ni18n_f};
use chrono::{Local, NaiveTime, Timelike};
use glib::source::timeout_add_seconds_local;
use gtk::{gio::subclass::prelude::*, glib};
use notify_rust::{Notification, Timeout, Urgency};
use std::{convert::TryInto, string::ToString};

#[derive(
    PartialEq,
    Debug,
    Clone,
    Copy,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum NotifyMode {
    Hourly,
    Every4hrs,
    Fixed,
}

impl Default for NotifyMode {
    fn default() -> Self {
        Self::Every4hrs
    }
}

mod imp {
    use crate::core::{Database, Settings};
    use gtk::{glib, subclass::prelude::*};

    #[derive(Debug, Default)]
    pub struct ModelNotificationMut {
        pub hour_count: i32,
    }

    #[derive(Debug, Default)]
    pub struct ModelNotification {
        pub database: Database,
        pub settings: Settings,
        pub inner: std::cell::RefCell<ModelNotificationMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModelNotification {
        const NAME: &'static str = "HealthModelNotification";
        type ParentType = glib::Object;
        type Type = super::ModelNotification;
    }

    impl ObjectImpl for ModelNotification {}
}

glib::wrapper! {
    /// The data belonging to a certain [ActivityTypeRow](crate::views::ActivityTypeRow).
    /// This can be fed to a [ActivityTypeRow](crate::views::ActivityTypeRow) via
    /// a [gio::ListModel].
    pub struct ModelNotification(ObjectSubclass<imp::ModelNotification>);
}

impl ModelNotification {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ModelNotification")
    }

    pub fn periodic_notify(&self) {
        let periodic = glib::clone!(@strong self as x => move || {
            glib::clone!(@weak x as obj => move || {
                gtk_macros::spawn!(
                    async move {
                        obj.periodic_callback().await;
                    }
                );
            })();
        });
        timeout_add_seconds_local(60, move || {
            periodic();
            glib::Continue(true)
        });
    }

    fn imp(&self) -> &imp::ModelNotification {
        imp::ModelNotification::from_instance(self)
    }

    async fn periodic_callback(&self) {
        self.imp().inner.borrow_mut().hour_count += 1;
        let time_now = Local::now().time();
        let notify_time =
            NaiveTime::parse_from_str(self.imp().settings.notification_time().as_str(), "%H:%M:%S")
                .unwrap();
        let interval = match self.imp().settings.notification_frequency() {
            NotifyMode::Hourly => 60,
            NotifyMode::Every4hrs => 60 * 4,
            NotifyMode::Fixed => 0,
        };
        let fixed_time = time_now.hour() == notify_time.hour()
            && time_now.minute() == notify_time.minute()
            && self.imp().settings.notification_frequency() == NotifyMode::Fixed;
        let periodic = self.imp().settings.notification_frequency() != NotifyMode::Fixed
            && self.imp().inner.borrow().hour_count % interval == 0;
        if (fixed_time || periodic) && self.imp().settings.enable_notifications() {
            Notification::new()
                .summary(&i18n("Health: walking reminder"))
                .body(&(self.reminder_text().await))
                .icon("dev.Cogitri.Health")
                .appname("Health")
                .timeout(Timeout::Milliseconds(2000))
                .urgency(Urgency::Low)
                .show()
                .unwrap();
        }
    }

    async fn reminder_text(&self) -> String {
        let stepgoal = i64::from(self.imp().settings.user_stepgoal());
        let stepcount = self
            .imp()
            .database
            .todays_steps(chrono::Local::today().and_hms(0, 0, 0).into())
            .await
            .unwrap();
        let message_pool = vec![ni18n_f(
            "{} step remaining to complete your daily step goal of {} steps",
            "{} steps remaining to complete your daily step goal of {} steps",
            (stepgoal - stepcount).try_into().unwrap_or(0),
            &[&(stepgoal - stepcount).to_string(), &stepgoal.to_string()],
        )];
        message_pool[0].clone()
    }
}
