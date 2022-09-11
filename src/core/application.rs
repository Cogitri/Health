/* application.rs
 *
 * Copyright 2020-2022 Rasmus Thomsen <oss@cogitri.dev>
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
    core::{i18n, Database, UnitSystem},
    model::{ModelNotification, User},
    windows::{PreferencesWindow, Window},
};
use anyhow::Result;
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
        core::{Database, Settings},
        model::ModelNotification,
        windows::{SetupWindow, Window},
    };
    use adw::subclass::prelude::*;
    use gtk::{
        gio,
        glib::{self, clone, g_warning},
    };
    use gtk::{prelude::*, subclass::prelude::*};
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct Application {
        pub notification_model: RefCell<Option<ModelNotification>>,
        pub settings: Settings,
        pub database: Database,
        pub window: RefCell<Option<glib::WeakRef<Window>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "HealthApplication";
        type ParentType = adw::Application;
        type Type = super::Application;
    }

    impl ObjectImpl for Application {}
    impl ApplicationImpl for Application {
        fn activate(&self, obj: &Self::Type) {
            self.parent_activate(obj);

            if let Some(window) = self.window.borrow().clone().and_then(|s| s.upgrade()) {
                window.present();
                return;
            }

            if self.settings.active_user_id() > 0 || self.settings.did_initial_setup() {
                glib::g_info!(
                    crate::config::APPLICATION_ID,
                    "Migrating DB and starting main window..."
                );

                self.settings.set_did_initial_setup(false);
                // Make sure we don't exit while the migration is running because we haven't opened a window yet
                obj.hold();
                gtk_macros::spawn!(glib::clone!(@weak obj => async move {
                    obj.create_main_window().await;
                }));
            } else {
                glib::g_info!(crate::config::APPLICATION_ID, "Starting setup...");

                let setup_window = SetupWindow::new(obj);
                obj.hold();

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

            obj.setup_actions();
            obj.setup_accels();

            if obj.flags().contains(gio::ApplicationFlags::IS_SERVICE) {
                if self.settings.active_user_id() > 0 {
                    gtk_macros::spawn!(glib::clone!(@weak obj => async move {
                        obj.setup_notifications().await;
                    }));
                }

                // Hold onto this application to send notifications
                obj.hold();
            }
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

    pub fn handle_shutdown(&self, check_windows: bool) {
        // Only actually quit here if background notifications aren't enabled
        if !self.flags().contains(gio::ApplicationFlags::IS_SERVICE)
            && (!check_windows || self.windows().is_empty())
        {
            self.quit();
        }
    }

    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::config::APPLICATION_ID),
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
                    .borrow()
                    .clone()
                    .and_then(|s| s.upgrade())
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

    pub async fn get_user(&self) -> User {
        let imp = self.imp();
        let user_id = i64::from(imp.settings.active_user_id());
        imp.database.user(user_id).await.unwrap()
    }

    async fn create_main_window(&self) {
        if let Err(e) = Database::instance().migrate().await {
            glib::g_warning!(
                crate::config::APPLICATION_ID,
                "Failed to migrate database to new version due to error {e}",
            );
        }
        let window = Window::new(self);
        window.show();
        self.imp()
            .window
            .replace(Some(glib::ObjectExt::downgrade(&window)));
        // Since the window is shown now we can release the hold, the application will exit once the window is closed (if notifications are disabled)
        self.release();
    }

    async fn handle_enable_notifications_changed(&self) {
        let imp = self.imp();
        let user = self.get_user().await;
        if imp.settings.enable_notifications() {
            let model = ModelNotification::new(
                self,
                imp.settings.notification_frequency(),
                imp.settings.notification_time(),
                user.user_stepgoal().unwrap() as u32,
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
                .borrow()
                .clone()
                .and_then(|s| s.upgrade())
                .map(glib::Cast::upcast),
        )
        .show()
    }

    fn handle_quit(&self) {
        if let Some(window) = self.imp().window.borrow().clone().and_then(|s| s.upgrade()) {
            window.destroy();
        }

        self.handle_shutdown(true)
    }

    fn handle_setup_window_setup_done(&self) {
        let imp = self.imp();
        let window = Window::new(self);
        window.show();
        imp.window
            .replace(Some(glib::ObjectExt::downgrade(&window)));
        self.release();
    }

    fn handle_unit_system(&self, action: &gio::SimpleAction, parameter: Option<&glib::Variant>) {
        let parameter = parameter.unwrap();

        self.imp().settings.set_unit_system(
            UnitSystem::from_str(parameter.to_string().replace('\'', "").as_str()).unwrap(),
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

    fn setup_accels(&self) {
        self.set_accels_for_action("win.fullscreen", &["F11"]);
        self.set_accels_for_action("win.hamburger-menu", &["F10"]);
        self.set_accels_for_action("app.help", &["F1"]);
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

    async fn setup_notifications(&self) {
        let imp = self.imp();
        let user = self.get_user().await;

        self.handle_enable_notifications_changed().await;

        imp.settings.connect_enable_notifications_changed(
            clone!(@weak self as obj => move |_, _| {
                 glib::MainContext::default().spawn_local(async move {
                    obj.handle_enable_notifications_changed().await;
                });
            }),
        );

        imp.database
            .connect_user_updated(clone!(@weak self as obj => move |_| {
                let imp = obj.imp();
                if let Some(model) = &*imp.notification_model.borrow() {
                    model.set_step_goal(user.user_stepgoal().unwrap_or(0) as u32);
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
    use crate::core::utils::init_gschema;

    #[test]
    fn new() {
        let _tmp = init_gschema();
        Application::new();
    }
}
