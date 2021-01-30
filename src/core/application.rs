use crate::{config, core::settings::HealthSettings};
use gtk::{gio, glib};

mod imp {
    use super::*;
    use crate::{
        core::{i18n, HealthDatabase},
        windows::{HealthPreferencesWindow, HealthSetupWindow, HealthWindow},
    };
    use gio::ActionMapExt;
    use glib::{clone, g_warning, subclass};
    use gtk::{prelude::*, subclass::prelude::*};
    use gtk_macros::action;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct HealthApplication {
        pub db: HealthDatabase,
        pub settings: HealthSettings,
        pub window: RefCell<glib::WeakRef<HealthWindow>>,
    }

    impl ObjectSubclass for HealthApplication {
        const NAME: &'static str = "HealthApplication";
        type ParentType = gtk::Application;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthApplication;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                db: HealthDatabase::new().expect("Failed to connect to Tracker Database!"),
                settings: HealthSettings::new(),
                window: RefCell::new(glib::WeakRef::new()),
            }
        }

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self::Type>) {}
    }

    impl ObjectImpl for HealthApplication {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ApplicationImpl for HealthApplication {
        fn activate(&self, application: &Self::Type) {
            self.parent_activate(application);

            if self.window.borrow().upgrade().is_some() {
                return;
            } else if self.settings.get_did_initial_setup() {
                let window = HealthWindow::new(application, self.db.clone());
                window.show();
                self.window.replace(glib::ObjectExt::downgrade(&window));
            } else {
                let setup_window = HealthSetupWindow::new(application);
                setup_window.show();
            }
        }

        fn startup(&self, application: &Self::Type) {
            self.parent_startup(application);
            adw::init();

            if let Some(true) = gtk::Settings::get_default()
                .and_then(|s| s.get_property_gtk_theme_name())
                .map(|s| s.as_str().contains("-dark"))
            {
                g_warning! (config::LOG_DOMAIN, "Using -dark themes (such as Adwaita-dark) is unsupported. Please use your theme in dark-mode instead (e.g. Adwaita:dark instead of Adwaita-dark)");
            }

            self.setup_actions(application);
            self.setup_accels(application);
        }
    }
    impl GtkApplicationImpl for HealthApplication {}

    impl HealthApplication {
        fn setup_actions(&self, obj: &super::HealthApplication) {
            action!(obj, "about", move |_, _| {
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
                obj,
                "fullscreen",
                clone!(@weak obj => move |_, _| {
                    if let Some(window) = imp::HealthApplication::from_instance(&obj).window.borrow().upgrade() {
                        if window.is_fullscreen() {
                            window.unfullscreen();
                        } else {
                            window.fullscreen();
                        }
                    }
                })
            );

            action!(
                obj,
                "hamburger-menu",
                clone!(@weak obj => move |_, _| {
                    if let Some(window) = imp::HealthApplication::from_instance(&obj).window.borrow().upgrade() {
                        window.open_hamburger_menu();
                    }
                })
            );

            action!(
                obj,
                "help",
                clone!(@weak obj => move |_, _| {
                })
            );

            action!(
                obj,
                "preferences",
                clone!(@weak obj => move |_, _| {
                    let self_ = imp::HealthApplication::from_instance(&obj);
                    let preferences_window = HealthPreferencesWindow::new(self_.db.clone(), self_.window.borrow().upgrade().map(|w| w.upcast()));
                    preferences_window.show();
                })
            );

            action!(
                obj,
                "quit",
                clone!(@weak obj => move |_, _| {
                    if let Some(window) = imp::HealthApplication::from_instance(&obj).window.borrow().upgrade() {
                        window.destroy();
                    }
                })
            );

            action!(obj, "shortcuts", move |_, _| {
                gtk::Builder::from_resource("/dev/Cogitri/Health/ui/shortcuts_window.ui")
                    .get_object::<gtk::ShortcutsWindow>("shortcuts_window")
                    .unwrap()
                    .show();
            });
        }

        fn setup_accels(&self, obj: &super::HealthApplication) {
            obj.set_accels_for_action("app.fullscreen", &["F11"]);
            obj.set_accels_for_action("app.hamburger-menu", &["F10"]);
            obj.set_accels_for_action("app.help", &["F1"]);
            obj.set_accels_for_action("app.quit", &["<Primary>q"]);
            obj.set_accels_for_action("app.shortcuts", &["<Primary>question"]);
        }
    }
}

glib::wrapper! {
    pub struct HealthApplication(ObjectSubclass<imp::HealthApplication>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl HealthApplication {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APPLICATION_ID.to_string()),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
        ])
        .expect("Failed to create HealthApplication")
    }
}
