/* model_notification.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
 * Copyright 2022 Rasmus Thomsen <oss@cogitri.dev>
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

use crate::{
    core::{i18n, ni18n_f},
    model::NotificationFrequency,
    prelude::*,
};
use chrono::{Local, Timelike};
use gtk::{
    gio::{self, prelude::*, subclass::prelude::*},
    glib,
};
use std::{convert::TryInto, string::ToString};

mod imp {
    use crate::{core::Database, model::NotificationFrequency, prelude::*};
    use chrono::NaiveTime;
    use gtk::{
        gio::{self, prelude::*, subclass::prelude::*},
        glib,
    };
    use once_cell::unsync::OnceCell;
    use std::{cell::RefCell, str::FromStr};

    #[derive(Debug, Default)]
    pub struct ModelNotificationMut {
        pub hour_count: i32,
        pub notification_frequency: NotificationFrequency,
        pub notification_time: Option<NaiveTime>,
        pub step_goal: u32,
        pub timeout_source_id: Option<glib::SourceId>,
    }

    #[derive(Debug, Default)]
    pub struct ModelNotification {
        pub application: OnceCell<gio::Application>,
        pub database: Database,
        pub inner: RefCell<ModelNotificationMut>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModelNotification {
        const NAME: &'static str = "HealthModelNotification";
        type ParentType = glib::Object;
        type Type = super::ModelNotification;
    }

    impl ObjectImpl for ModelNotification {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "application",
                        "application",
                        "application",
                        gio::Application::static_type(),
                        glib::ParamFlags::CONSTRUCT_ONLY | glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "notification-frequency",
                        "notification-frequency",
                        "notification-frequency",
                        Some(NotificationFrequency::default().as_ref()),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
                        "notification-time",
                        "notification-time",
                        "notification-time",
                        NaiveTimeBoxed::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
                    ),
                    glib::ParamSpecUInt::new(
                        "step-goal",
                        "step-goal",
                        "step-goal",
                        0,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "application" => self.application.set(value.get().unwrap()).unwrap(),
                "notification-frequency" => {
                    self.inner.borrow_mut().notification_frequency =
                        NotificationFrequency::from_str(value.get().unwrap()).unwrap()
                }
                "notification-time" => {
                    self.inner.borrow_mut().notification_time =
                        Some(value.get::<NaiveTimeBoxed>().unwrap().0);
                }
                "step-goal" => self.inner.borrow_mut().step_goal = value.get().unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "application" => self.application.get().unwrap().to_value(),
                "notification-frequency" => self
                    .inner
                    .borrow()
                    .notification_frequency
                    .as_ref()
                    .to_value(),
                "notification-time" => {
                    NaiveTimeBoxed(self.inner.borrow().notification_time.unwrap()).to_value()
                }
                "step-goal" => self.inner.borrow().step_goal.to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// The data belonging to a certain [ActivityTypeRow](crate::views::ActivityTypeRow).
    /// This can be fed to a [ActivityTypeRow](crate::views::ActivityTypeRow) via
    /// a [gio::ListModel].
    pub struct ModelNotification(ObjectSubclass<imp::ModelNotification>);
}

impl ModelNotification {
    pub fn new<T: IsA<gio::Application>>(
        application: &T,
        notification_frequency: NotificationFrequency,
        notification_time: chrono::NaiveTime,
        step_goal: u32,
    ) -> Self {
        glib::Object::new(&[
            ("application", application),
            ("notification-frequency", &notification_frequency),
            ("notification-time", &NaiveTimeBoxed(notification_time)),
            ("step-goal", &step_goal),
        ])
        .expect("Failed to create ModelNotification")
    }

    pub fn register_periodic_notify(&self) {
        let source_id = glib::source::timeout_add_seconds_local(
            60,
            glib::clone!(@strong self as obj => move || {
                gtk_macros::spawn!(glib::clone!(@weak obj => async move {
                    obj.periodic_callback().await;
                }));

                glib::Continue(true)
            }),
        );

        self.imp().inner.borrow_mut().timeout_source_id = Some(source_id);
    }

    pub fn set_notification_frequency(&self, value: NotificationFrequency) {
        self.set_property("notification-frequency", value);
    }

    pub fn set_notification_time(&self, value: chrono::NaiveTime) {
        self.set_property("notification-time", NaiveTimeBoxed(value));
    }

    pub fn set_step_goal(&self, value: u32) {
        self.set_property("step-goal", value);
    }

    pub fn step_goal(&self) -> u32 {
        self.property("step-goal")
    }

    pub fn unregister_periodic_notify(&self) {
        if let Some(id) = self.imp().inner.borrow_mut().timeout_source_id.take() {
            id.remove();
        }
    }

    async fn periodic_callback(&self) {
        let imp = self.imp();
        let time_now = Local::now().time();
        let notify_time = imp.inner.borrow().notification_time.unwrap();
        let frequency = imp.inner.borrow().notification_frequency;

        if time_now.minute() == 0 {
            imp.inner.borrow_mut().hour_count += 1;
        }

        let interval = match frequency {
            NotificationFrequency::Hourly => 60,
            NotificationFrequency::Every4Hrs => 60 * 4,
            NotificationFrequency::Fixed => 0,
        };
        let fixed_time = time_now.hour() == notify_time.hour()
            && time_now.minute() == notify_time.minute()
            && frequency == NotificationFrequency::Fixed;
        let periodic = frequency != NotificationFrequency::Fixed
            && imp.inner.borrow().hour_count % interval == 0
            && time_now.minute() == 0;
        if fixed_time || periodic {
            let notification = gio::Notification::new(&i18n("Health: walking reminder"));
            notification.set_body(Some(&(self.reminder_text().await)));
            notification.set_icon(&gio::Icon::for_string(crate::config::APPLICATION_ID).unwrap());
            imp.application
                .get()
                .unwrap()
                .send_notification(Some("walking-reminder"), &notification);
        }
    }

    // TRANSLATORS notes have to be on the same line, so we cant split them
    #[rustfmt::skip::attributes(ni18n_f)]
    async fn reminder_text(&self) -> String {
        let step_goal = i64::from(self.imp().inner.borrow().step_goal);
        let step_count = self
            .imp()
            .database
            .todays_steps(chrono::Local::today().and_hms(0, 0, 0).into())
            .await
            .unwrap();
        let message_pool = vec![{
            // TRANSLATORS: First part of message, ends with [...] of {} steps[.] See next source string.
            ni18n_f("{} step remaining to complete your daily step goal",
                "{} steps remaining to complete your daily step goal",
                (step_goal - step_count).try_into().unwrap_or(0),
                &[&(step_goal - step_count).to_string()],
            ) +
            // TRANSLATORS: Second (final) part of message, see previous source string.
            &ni18n_f("of {} step",
                "of {} steps",
                step_goal.try_into().unwrap(),
                &[&step_goal.to_string()],
            )
        }];
        message_pool[0].clone()
    }
}

#[cfg(test)]
mod test {
    use super::{ModelNotification, NotificationFrequency};
    use gtk::gio;

    #[test]
    fn new() {
        ModelNotification::new(
            &gio::Application::new(None, gio::ApplicationFlags::FLAGS_NONE),
            NotificationFrequency::Every4Hrs,
            chrono::NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap(),
            1000,
        );
    }
}
