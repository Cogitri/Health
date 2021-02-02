use crate::config;
use gtk::{gio, glib};

mod imp {
    use crate::{
        config,
        core::{i18n, Database, Settings},
        windows::{Window, PreferencesWindow, SetupWindow},
    };
    use gio::ActionMapExt;
    use glib::{clone, g_warning, subclass};
    use gtk::{prelude::*, subclass::prelude::*};
    use gtk_macros::action;
    use once_cell::unsync::OnceCell;

    #[derive(Debug)]
    pub struct Application {
        pub db: Database,
        pub settings: Settings,
        pub window: OnceCell<glib::WeakRef<Window>>,
    }

    impl ObjectSubclass for Application {
        const NAME: &'static str = "HealthApplication";
        type ParentType = gtk::Application;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::Application;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                db: Database::new().expect("Failed to connect to Tracker Database!"),
                settings: Settings::new(),
                window: OnceCell::new(),
            }
        }

        fn class_init(_klass: &mut Self::Class) {}

        fn instance_init(_obj: &glib::subclass::InitializingObject<Self::Type>) {}
    }

    impl ObjectImpl for Application {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ApplicationImpl for Application {
        fn activate(&self, application: &Self::Type) {
            self.parent_activate(application);
            let has_window = self.window.get().and_then(glib::WeakRef::upgrade).is_some();

            if !has_window && self.settings.get_did_initial_setup() {
                let window = Window::new(application, self.db.clone());
                window.show();
                self.window
                    .set(glib::ObjectExt::downgrade(&window))
                    .unwrap();
            } else if !has_window {
                let setup_window = SetupWindow::new(application, self.db.clone());

                setup_window.connect_setup_done(clone!(@weak application => move || {
                    let self_ = Application::from_instance(&application);
                    self_.settings.set_did_initial_setup(true);
                    let window = Window::new(&application, self_.db.clone());
                    window.show();
                    self_.window
                        .set(glib::ObjectExt::downgrade(&window))
                        .unwrap();
                }));

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
    impl GtkApplicationImpl for Application {}

    impl Application {
        fn setup_actions(&self, obj: &super::Application) {
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
                    if let Some(window) = Application::from_instance(&obj).window.get().and_then(|w| w.upgrade()) {
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
                    if let Some(window) = Application::from_instance(&obj).window.get().and_then(|w| w.upgrade()) {
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
                    let self_ = Application::from_instance(&obj);
                    let preferences_window = PreferencesWindow::new(self_.db.clone(), self_.window.get().and_then(|w| w.upgrade()).map(|w| w.upcast()));
                    preferences_window.show();
                })
            );

            action!(
                obj,
                "quit",
                clone!(@weak obj => move |_, _| {
                    if let Some(window) = Application::from_instance(&obj).window.get().and_then(|w| w.upgrade()) {
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

        fn setup_accels(&self, obj: &super::Application) {
            obj.set_accels_for_action("app.fullscreen", &["F11"]);
            obj.set_accels_for_action("app.hamburger-menu", &["F10"]);
            obj.set_accels_for_action("app.help", &["F1"]);
            obj.set_accels_for_action("app.quit", &["<Primary>q"]);
            obj.set_accels_for_action("app.shortcuts", &["<Primary>question"]);
        }
    }
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new(&[
            ("application-id", &config::APPLICATION_ID.to_string()),
            ("flags", &gio::ApplicationFlags::FLAGS_NONE),
        ])
        .expect("Failed to create Application")
    }
}
