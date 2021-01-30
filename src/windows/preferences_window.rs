use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use std::cell::RefCell;

    use super::*;
    use crate::{
        core::{i18n, settings::Unitsystem, utils::get_spinbutton_value, HealthSettings},
        widgets::{HealthBMILevelBar, HealthSyncListBox},
    };
    use adw::PreferencesRowExt;
    use glib::{
        clone,
        subclass::{self, Signal},
    };
    use gtk::subclass::prelude::*;
    use uom::si::{
        f32::{Length, Mass},
        length::{centimeter, inch},
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    pub struct HealthPreferencesWindow {
        pub parent_window: RefCell<Option<gtk::Window>>,
        pub settings: HealthSettings,

        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weightgoal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub age_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub height_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub stepgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub weightgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub unit_imperial_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub unit_metric_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub bmi_levelbar: TemplateChild<HealthBMILevelBar>,
        #[template_child]
        pub sync_list_box: TemplateChild<HealthSyncListBox>,
    }

    impl ObjectSubclass for HealthPreferencesWindow {
        const NAME: &'static str = "HealthPreferencesWindow";
        type ParentType = adw::PreferencesWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthPreferencesWindow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: HealthSettings::new(),
                height_actionrow: TemplateChild::default(),
                weightgoal_actionrow: TemplateChild::default(),
                age_spin_button: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                stepgoal_spin_button: TemplateChild::default(),
                weightgoal_spin_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                bmi_levelbar: TemplateChild::default(),
                parent_window: RefCell::new(None),
                sync_list_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HealthPreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.settings.get_unitsystem() == Unitsystem::Metric {
                self.unit_metric_togglebutton.set_active(true);
                self.height_actionrow
                    .set_title(Some(&i18n("Height in centimeters")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in KG")));
                self.height_spin_button
                    .set_value(self.settings.get_user_height().get::<centimeter>() as f64);
                self.weightgoal_spin_button
                    .set_value(self.settings.get_user_weightgoal().get::<kilogram>() as f64);
            } else {
                self.unit_metric_togglebutton.set_active(true);
                self.height_actionrow
                    .set_title(Some(&i18n("Height in inch")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in pounds")));
                self.height_spin_button
                    .set_value(self.settings.get_user_height().get::<inch>() as f64);
                self.weightgoal_spin_button
                    .set_value(self.settings.get_user_weightgoal().get::<pound>() as f64);
            }

            self.stepgoal_spin_button
                .set_value(self.settings.get_user_stepgoal() as f64);
            self.age_spin_button
                .set_value(self.settings.get_user_age() as f64);

            self.connect_handlers(obj);
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("import-done", &[], glib::Type::Unit).build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for HealthPreferencesWindow {}
    impl WindowImpl for HealthPreferencesWindow {}
    impl adw::subclass::window::AdwWindowImpl for HealthPreferencesWindow {}
    impl adw::subclass::preferences_window::PreferencesWindowImpl for HealthPreferencesWindow {}

    impl HealthPreferencesWindow {
        fn connect_handlers(&self, obj: &super::HealthPreferencesWindow) {
            self.age_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = imp::HealthPreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.age_spin_button);
                    if val != 0 {
                        self_.settings.set_user_age(val);
                    }
                }));

            self.stepgoal_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = imp::HealthPreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.stepgoal_spin_button);
                    if val != 0 {
                        self_.settings.set_user_stepgoal(val);
                    }
                }));

            self.weightgoal_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = imp::HealthPreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<f32>(&self_.weightgoal_spin_button);
                    if val != 0.0 {
                        let weight = if self_.unit_metric_togglebutton.get_active() {
                            Mass::new::<kilogram>(val)
                        } else {
                            Mass::new::<pound>(val)
                        };

                        self_.settings.set_user_weightgoal(weight);
                    }
                }));

            self.height_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = imp::HealthPreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.height_spin_button) as f32;
                    if val != 0.0 {
                        let height = if self_.unit_metric_togglebutton.get_active() {
                            Length::new::<centimeter>(val)
                        } else {
                            Length::new::<inch>(val)
                        };

                        self_.settings.set_user_height(height);
                    }
                }));

            self.unit_metric_togglebutton.connect_toggled(clone!(@weak obj => move |btn| {
                let self_ = imp::HealthPreferencesWindow::from_instance(&obj);
                if btn.get_active() {
                    self_.settings.set_unitsystem(Unitsystem::Metric);
                    self_.bmi_levelbar.set_unitsystem(Unitsystem::Metric);
                    self_.height_actionrow
                    .set_title(Some(&i18n("Height in centimeters")));
                    self_.weightgoal_actionrow
                        .set_title(Some(&i18n("Weightgoal in KG")));
                    self_.height_spin_button
                        .set_value(Length::new::<inch>(get_spinbutton_value(&self_.height_spin_button)).get::<centimeter>() as f64);
                    self_.weightgoal_spin_button
                        .set_value(Mass::new::<pound>(get_spinbutton_value(&self_.height_spin_button)).get::<kilogram>() as f64);
                } else {
                    self_.settings.set_unitsystem(Unitsystem::Imperial);
                    self_.bmi_levelbar.set_unitsystem(Unitsystem::Imperial);
                    self_.height_actionrow
                    .set_title(Some(&i18n("Height in inch")));
                    self_.weightgoal_actionrow
                        .set_title(Some(&i18n("Weightgoal in pounds")));
                    self_.height_spin_button
                        .set_value(Length::new::<centimeter>(get_spinbutton_value(&self_.height_spin_button)).get::<inch>() as f64);
                    self_.weightgoal_spin_button
                        .set_value(Mass::new::<kilogram>(get_spinbutton_value(&self_.height_spin_button)).get::<pound>() as f64);
                }
            }));

        }
    }
}

glib::wrapper! {
    pub struct HealthPreferencesWindow(ObjectSubclass<imp::HealthPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::PreferencesWindow;
}

impl HealthPreferencesWindow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create HealthPreferencesWindow")
    }

    pub fn set_parent_window(&self, w: Option<gtk::Window>) {
        imp::HealthPreferencesWindow::from_instance(&self).set_parent_window(w);
    }

    pub fn connect_import_done<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("import-done", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }
}
