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
    core::{i18n, settings::Unitsystem},
    windows::{PreferencesWindow, Window},
};
use gio::prelude::*;
use glib::{clone, subclass::prelude::*};
use gtk::prelude::*;
use gtk_macros::{action, stateful_action};
use std::convert::TryFrom;

mod imp {
    use crate::{
        config,
        core::Settings,
        windows::{SetupWindow, Window},
    };
    use glib::{clone, g_warning};
    use gtk::{prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;

    #[derive(Debug)]
    pub struct Application {
        pub settings: Settings,
        pub window: OnceCell<glib::WeakRef<Window>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "HealthApplication";
        type ParentType = gtk::Application;
        type Type = super::Application;

        fn new() -> Self {
            Self {
                settings: Settings::get_instance(),
                window: OnceCell::new(),
            }
        }

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self>) {}
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self, obj: &Self::Type) {
            self.parent_activate(obj);
            let has_window = self.window.get().and_then(glib::WeakRef::upgrade).is_some();

            if !has_window && self.settings.get_did_initial_setup() {
                let window = Window::new(obj);
                window.show();
                self.window
                    .set(glib::ObjectExt::downgrade(&window))
                    .unwrap();
            } else if !has_window {
                let setup_window = SetupWindow::new(obj);

                setup_window.connect_setup_done(clone!(@weak obj => move || {
                    obj.handle_setup_window_setup_done();
                }));

                setup_window.show();
            }
        }

        fn startup(&self, obj: &Self::Type) {
            self.parent_startup(obj);
            adw::init();

            if let Some(true) = gtk::Settings::get_default()
                .and_then(|s| s.get_property_gtk_theme_name())
                .map(|s| s.as_str().contains("-dark"))
            {
                g_warning! (config::LOG_DOMAIN, "Using -dark themes (such as Adwaita-dark) is unsupported. Please use your theme in dark-mode instead (e.g. Adwaita:dark instead of Adwaita-dark)");
            }

            obj.setup_actions();
            obj.setup_accels();
        }
    }
    impl GtkApplicationImpl for Application {}
}

glib::wrapper! {
    /// [Application] is an implementation of [GtkApplication](gtk::Application) that handles starting & managing the windows etc.
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &crate::config::APPLICATION_ID.to_string()),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
        ])
        .expect("Failed to create Application")
    }

    fn get_priv(&self) -> &imp::Application {
        imp::Application::from_instance(self)
    }

    fn handle_setup_window_setup_done(&self) {
        let self_ = self.get_priv();
        self_.settings.set_did_initial_setup(true);
        let window = Window::new(self);
        window.show();
        self_
            .window
            .set(glib::ObjectExt::downgrade(&window))
            .unwrap();
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.fullscreen", &["F11"]);
        self.set_accels_for_action("app.hamburger-menu", &["F10"]);
        self.set_accels_for_action("app.help", &["F1"]);
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
        self.set_accels_for_action("app.shortcuts", &["<Primary>question"]);
    }

    fn setup_actions(&self) {
        action!(self, "about", move |_, _| {
            gtk::AboutDialogBuilder::new()
                .logo_icon_name(crate::config::APPLICATION_ID)
                .program_name("Health")
                .comments(&i18n("A health tracking app for the GNOME desktop."))
                .authors(vec!["Rasmus Thomsen <oss@cogitri.dev>".to_string()])
                .translator_credits(&i18n("translator-credits"))
                .website("https://gitlab.gnome.org/Cogitri/gnome-health")
                .website_label(&i18n("Websites"))
                .version(crate::config::VERSION)
                .license_type(gtk::License::Gpl30)
                .build()
                .show()
        });
        action!(
            self,
            "fullscreen",
            clone!(@weak self as obj => move |_, _| {
                if let Some(window) = obj.get_priv().window.get().and_then(glib::WeakRef::upgrade) {
                    if window.is_fullscreen() {
                        window.unfullscreen();
                    } else {
                        window.fullscreen();
                    }
                }
            })
        );

        action!(
            self,
            "hamburger-menu",
            clone!(@weak self as obj => move |_, _| {
                if let Some(window) = obj.get_priv().window.get().and_then(glib::WeakRef::upgrade) {
                    window.open_hamburger_menu();
                }
            })
        );

        action!(
            self,
            "help",
            clone!(@weak self as obj => move |_, _| {
            })
        );

        action!(
            self,
            "preferences",
            clone!(@weak self as obj => move |_, _| {
                let self_ = obj.get_priv();
                let preferences_window = PreferencesWindow::new(self_.window.get().and_then(glib::WeakRef::upgrade).map(glib::Cast::upcast));
                preferences_window.show();
            })
        );

        action!(
            self,
            "quit",
            clone!(@weak self as obj => move |_, _| {
                if let Some(window) = obj.get_priv().window.get().and_then(glib::WeakRef::upgrade) {
                    window.destroy();
                }
            })
        );

        action!(self, "shortcuts", move |_, _| {
            gtk::Builder::from_resource("/dev/Cogitri/Health/ui/shortcuts_window.ui")
                .get_object::<gtk::ShortcutsWindow>("shortcuts_window")
                .unwrap()
                .show();
        });

        stateful_action!(
            self,
            "unitsystem",
            Some(&String::static_variant_type()),
            {
                let s: &str = self.get_priv().settings.get_unitsystem().into();
                s
            },
            clone!(@weak self as obj => move |a, p| {
                let parameter = p.unwrap();

                obj.get_priv().settings.set_unitsystem(Unitsystem::try_from(parameter.to_string().replace("'", "").as_str()).unwrap());

                a.set_state(parameter);
            })
        );
    }
}
