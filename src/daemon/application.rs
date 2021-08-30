/* application.rs
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
use gtk::{gio, glib};

mod imp {
    use crate::{model::ModelNotification, Settings};
    use gtk::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*};

    #[derive(Debug, Default)]
    pub struct Application {}

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "HealthDaemonApplication";
        type ParentType = gio::Application;

        type Type = super::Application;

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {}
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, obj: &Self::Type) {
            self.parent_activate(obj);

            if !Settings::instance().enable_notifications() {
                obj.release();
                return;
            }

            let model_notification = ModelNotification::new();
            model_notification.periodic_notify();
        }
    }
}

glib::wrapper! {
    /// [Application] is an implementation of [GioApplication](gio::Application) that handles displaying of notifications.
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::config::DAEMON_APPLICATION_ID),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
        ])
        .expect("Failed to create Application")
    }
}
