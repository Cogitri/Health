/* application.rs
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

use crate::{
    core::{i18n, UnitSystem},
    model::ModelNotification,
    windows::{PreferencesWindow, Window},
};
use anyhow::Result;
use chrono::{DateTime, Duration, FixedOffset, Local};
use gtk::{
    gio::{self, prelude::*},
    glib::{self, clone, subclass::prelude::*},
    prelude::*,
};
use gtk_macros::{action, stateful_action};
use std::{path::Path, str::FromStr};

mod imp {
    use crate::{
        config,
        core::Settings,
        model::ModelNotification,
        windows::{SetupWindow, Window},
    };
    use adw::subclass::prelude::*;
    use gtk::glib::{self, clone, g_warning};
    use gtk::{prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct Application {
        pub notification_model: RefCell<Option<ModelNotification>>,
        pub settings: Settings,
        pub window: OnceCell<glib::WeakRef<Window>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "HealthApplication";
        type ParentType = adw::Application;
        type Type = super::Application;

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {}
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, obj: &Self::Type) {
            self.parent_activate(obj);
            let has_window = self.window.get().and_then(glib::WeakRef::upgrade).is_some();

            if !has_window && self.settings.did_initial_setup() {
                let window = Window::new(obj);
                window.show();
                self.window
                    .set(glib::ObjectExt::downgrade(&window))
                    .unwrap();
            } else if !has_window {
                let setup_window = SetupWindow::new(obj);

                setup_window.connect_setup_done(clone!(@weak obj => move |_| {
                    obj.handle_setup_window_setup_done();
                }));

                setup_window.show();
            }
        }

        fn startup(&self, obj: &Self::Type) {
            obj.set_resource_base_path(Some("/dev/Cogitri/Health"));

            self.parent_startup(obj);

            if let Some(true) = gtk::Settings::default()
                .and_then(|s| s.gtk_theme_name())
                .map(|s| s.as_str().contains("-dark"))
            {
                g_warning! (config::LOG_DOMAIN, "Using -dark themes (such as Adwaita-dark) is unsupported. Please use your theme in dark-mode instead (e.g. Adwaita:dark instead of Adwaita-dark)");
            }

            gtk::IconTheme::for_display(&gtk::gdk::Display::default().unwrap())
                .add_resource_path("/dev/Cogitri/Health/icons");

            obj.migrate_gsettings();
            obj.setup_actions();
            obj.setup_accels();
            obj.setup_notifications();
            // Hold onto this application to send notifications
            obj.hold();
        }
    }

    impl GtkApplicationImpl for Application {}
    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    /// [Application] is an implementation of [GtkApplication](gtk::Application) that handles starting & managing the windows etc.
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    fn delete_autostart_file(&self) -> Result<()> {
        let autostart_desktop_file = Path::new(crate::config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let mut autostart_user_file = glib::user_config_dir();
        autostart_user_file.push("autostart");
        autostart_user_file.push(&desktop_file_name);

        std::fs::remove_file(autostart_user_file)?;

        Ok(())
    }

    pub fn handle_shutdown(&self) {
        // Only actually quit here if background notifications aren't enabled
        if !self.flags().contains(gio::ApplicationFlags::IS_SERVICE) && self.windows().is_empty() {
            self.quit();
        }
    }

    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::config::APPLICATION_ID.to_string()),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
        ])
        .expect("Failed to create Application")
    }

    fn handle_about(&self) {
        gtk::AboutDialog::builder()
            .transient_for(
                &self
                    .imp()
                    .window
                    .get()
                    .and_then(glib::WeakRef::upgrade)
                    .unwrap(),
            )
            .modal(true)
            .logo_icon_name(crate::config::APPLICATION_ID)
            .program_name("Health")
            .comments(&i18n("A health tracking app for the GNOME desktop."))
            .authors(vec!["Rasmus Thomsen <oss@cogitri.dev>".to_string()])
            .translator_credits(&i18n("translator-credits"))
            .website("https://gitlab.gnome.org/World/Health")
            .website_label(&i18n("Websites"))
            .version(crate::config::VERSION)
            .license_type(gtk::License::Gpl30)
            .build()
            .show()
    }

    fn handle_enable_notifications_changed(&self) {
        let imp = self.imp();
        if imp.settings.enable_notifications() {
            let model = ModelNotification::new(
                self,
                imp.settings.notification_frequency(),
                imp.settings.notification_time(),
                imp.settings.user_step_goal(),
            );
            model.register_periodic_notify();
            imp.notification_model.replace(Some(model));
            if let Err(e) = self.install_autostart_file() {
                glib::g_warning!(
                    crate::config::APPLICATION_ID,
                    "Couldn't install autostart file due to error {e}",
                );
            }
        } else {
            if let Some(model) = imp.notification_model.borrow_mut().take() {
                model.unregister_periodic_notify();
            }
            if let Err(e) = self.delete_autostart_file() {
                glib::g_debug!(
                    crate::config::APPLICATION_ID,
                    "Couldn't remove autostart file due to error {e}",
                );
            }
        }
    }

    fn handle_help(&self) {}

    fn handle_preferences(&self) {
        PreferencesWindow::new(
            self.imp()
                .window
                .get()
                .and_then(glib::WeakRef::upgrade)
                .map(glib::Cast::upcast),
        )
        .show()
    }

    fn handle_quit(&self) {
        if let Some(window) = self.imp().window.get().and_then(glib::WeakRef::upgrade) {
            window.destroy();
        }

        self.handle_shutdown()
    }

    fn handle_setup_window_setup_done(&self) {
        let imp = self.imp();
        imp.settings.set_did_initial_setup(true);
        let window = Window::new(self);
        window.show();
        imp.window.set(glib::ObjectExt::downgrade(&window)).unwrap();
    }

    fn handle_shortcuts() {
        gtk::Builder::from_resource("/dev/Cogitri/Health/ui/shortcuts_window.ui")
            .object::<gtk::ShortcutsWindow>("shortcuts_window")
            .unwrap()
            .show();
    }

    fn handle_unit_system(&self, action: &gio::SimpleAction, parameter: Option<&glib::Variant>) {
        let parameter = parameter.unwrap();

        self.imp().settings.set_unit_system(
            UnitSystem::from_str(parameter.to_string().replace("'", "").as_str()).unwrap(),
        );

        action.set_state(parameter);
    }

    fn install_autostart_file(&self) -> Result<()> {
        let autostart_desktop_file = Path::new(crate::config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let mut autostart_user_folder = glib::user_config_dir();
        autostart_user_folder.push("autostart");

        if !autostart_user_folder.exists() {
            std::fs::create_dir_all(&autostart_user_folder)?;
        }
        autostart_user_folder.push(&desktop_file_name);
        if !autostart_user_folder.exists() {
            std::fs::copy(autostart_desktop_file, autostart_user_folder)?;
        }

        Ok(())
    }

    fn migrate_gsettings(&self) {
        let imp = self.imp();
        if imp.settings.user_birthday().is_none() {
            let age = imp.settings.user_age();
            let datetime: DateTime<FixedOffset> =
                (Local::now() - Duration::weeks((age * 52).into())).into();
            imp.settings.set_user_birthday(datetime.date());
        }
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("win.fullscreen", &["F11"]);
        self.set_accels_for_action("win.hamburger-menu", &["F10"]);
        self.set_accels_for_action("app.help", &["F1"]);
        self.set_accels_for_action("app.shortcuts", &["<Primary>question"]);
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
    }

    fn setup_actions(&self) {
        action!(
            self,
            "about",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_about();
            })
        );

        action!(
            self,
            "help",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_help();
            })
        );

        action!(
            self,
            "preferences",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_preferences();
            })
        );

        action!(
            self,
            "quit",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_quit();
            })
        );

        action!(self, "shortcuts", move |_, _| {
            Self::handle_shortcuts();
        });

        stateful_action!(
            self,
            "unit-system",
            Some(&String::static_variant_type()),
            self.imp().settings.unit_system().as_ref(),
            clone!(@weak self as obj => move |a, p| {
                obj.handle_unit_system(a, p);
            })
        );
    }

    fn setup_notifications(&self) {
        let imp = self.imp();

        self.handle_enable_notifications_changed();

        imp.settings.connect_enable_notifications_changed(
            clone!(@weak self as obj =>  move |_, _| {
                obj.handle_enable_notifications_changed();
            }),
        );

        imp.settings
            .connect_user_step_goal_changed(clone!(@weak self as obj => move |_, _| {
                let imp = obj.imp();
                if let Some(model) = &*imp.notification_model.borrow() {
                    model.set_step_goal(imp.settings.user_step_goal());
                };
            }));
        imp.settings.connect_notification_frequency_changed(
            clone!(@weak self as obj => move |_, _| {
                let imp = obj.imp();
                if let Some(model) = &*imp.notification_model.borrow() {
                    model.set_notification_frequency(imp.settings.notification_frequency())
                };
            }),
        );
        imp.settings
            .connect_notification_time_changed(clone!(@weak self as obj => move |_, _| {
                let imp = obj.imp();
                if let Some(model) = &*imp.notification_model.borrow() {
                    model.set_notification_time(imp.settings.notification_time())
                };
            }));
    }
}

#[cfg(test)]
mod test {
    use super::Application;
    use crate::core::{utils::init_gschema, Settings};
    use chrono::{Duration, Utc};

    #[test]
    fn new() {
        Application::new();
    }

    #[test]
    fn migrate_gsettings() {
        let _tmp = init_gschema();

        let app = Application::new();
        let settings = Settings::instance();
        settings.set_user_age(50);
        assert_eq!(settings.user_birthday(), None);
        app.migrate_gsettings();
        assert_eq!(
            settings
                .user_birthday()
                .unwrap()
                .format("%Y-%m-%d")
                .to_string(),
            (Utc::now() - Duration::weeks(50 * 52))
                .date()
                .format("%Y-%m-%d")
                .to_string(),
        );
    }
}
