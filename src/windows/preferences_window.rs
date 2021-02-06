/* preferences_window.rs
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

use crate::core::Database;
use glib::subclass::types::ObjectSubclass;
use gtk::prelude::*;

mod imp {
    use crate::{
        core::{i18n, settings::Unitsystem, utils::get_spinbutton_value, Database, Settings},
        sync::csv::CSVHandler,
        widgets::{BMILevelBar, SyncListBox},
    };
    use adw::PreferencesRowExt;
    use glib::{
        clone, g_warning,
        subclass::{self, Signal},
    };
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use gtk_macros::spawn;
    use once_cell::unsync::OnceCell;
    use uom::si::{
        f32::{Length, Mass},
        length::{centimeter, inch},
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        pub db: OnceCell<Database>,
        pub parent_window: OnceCell<Option<gtk::Window>>,
        pub settings: Settings,

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
        pub bmi_levelbar: TemplateChild<BMILevelBar>,
        #[template_child]
        pub sync_list_box: TemplateChild<SyncListBox>,
        #[template_child]
        pub export_activity_csv_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub export_weight_csv_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_activity_csv_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_weight_csv_button: TemplateChild<gtk::Button>,
    }

    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "HealthPreferencesWindow";
        type ParentType = adw::PreferencesWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::PreferencesWindow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: Settings::new(),
                height_actionrow: TemplateChild::default(),
                weightgoal_actionrow: TemplateChild::default(),
                age_spin_button: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                stepgoal_spin_button: TemplateChild::default(),
                weightgoal_spin_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                bmi_levelbar: TemplateChild::default(),
                parent_window: OnceCell::new(),
                db: OnceCell::new(),
                sync_list_box: TemplateChild::default(),
                export_activity_csv_button: TemplateChild::default(),
                export_weight_csv_button: TemplateChild::default(),
                import_activity_csv_button: TemplateChild::default(),
                import_weight_csv_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.settings.get_unitsystem() == Unitsystem::Metric {
                self.unit_metric_togglebutton.set_active(true);
                self.height_actionrow
                    .set_title(Some(&i18n("Height in centimeters")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in KG")));
                self.height_spin_button.set_value(f64::from(
                    self.settings.get_user_height().get::<centimeter>(),
                ));
                self.weightgoal_spin_button.set_value(f64::from(
                    self.settings.get_user_weightgoal().get::<kilogram>(),
                ));
            } else {
                self.unit_metric_togglebutton.set_active(true);
                self.height_actionrow
                    .set_title(Some(&i18n("Height in inch")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in pounds")));
                self.height_spin_button
                    .set_value(f64::from(self.settings.get_user_height().get::<inch>()));
                self.weightgoal_spin_button.set_value(f64::from(
                    self.settings.get_user_weightgoal().get::<pound>(),
                ));
            }

            self.stepgoal_spin_button
                .set_value(f64::from(self.settings.get_user_stepgoal()));
            self.age_spin_button
                .set_value(f64::from(self.settings.get_user_age()));

            self.bmi_levelbar
                .set_height(self.settings.get_user_height());

            self.bmi_levelbar
                .set_weight(self.settings.get_user_weightgoal());
            self.connect_handlers(obj);
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("import-done", &[], glib::Type::Unit).build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl adw::subclass::window::AdwWindowImpl for PreferencesWindow {}
    impl adw::subclass::preferences_window::PreferencesWindowImpl for PreferencesWindow {}

    impl PreferencesWindow {
        fn connect_handlers(&self, obj: &super::PreferencesWindow) {
            self.age_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = PreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.age_spin_button);
                    if val != 0 {
                        self_.settings.set_user_age(val);
                    }
                }));

            self.stepgoal_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = PreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.stepgoal_spin_button);
                    if val != 0 {
                        self_.settings.set_user_stepgoal(val);
                    }
                }));

            self.weightgoal_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = PreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<f32>(&self_.weightgoal_spin_button);
                    if val != 0.0 {
                        let weight = if self_.unit_metric_togglebutton.get_active() {
                            Mass::new::<kilogram>(val)
                        } else {
                            Mass::new::<pound>(val)
                        };

                        self_.settings.set_user_weightgoal(weight);
                        self_.bmi_levelbar.set_weight(weight);
                    }
                }));

            self.height_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = PreferencesWindow::from_instance(&obj);
                    let val = get_spinbutton_value::<u32>(&self_.height_spin_button) as f32;
                    if val != 0.0 {
                        let height = if self_.unit_metric_togglebutton.get_active() {
                            Length::new::<centimeter>(val)
                        } else {
                            Length::new::<inch>(val)
                        };

                        self_.settings.set_user_height(height);
                        self_.bmi_levelbar.set_height(height);
                    }
                }));

            self.unit_metric_togglebutton.connect_toggled(clone!(@weak obj => move |btn| {
                let self_ = PreferencesWindow::from_instance(&obj);
                if btn.get_active() {
                    self_.settings.set_unitsystem(Unitsystem::Metric);
                    self_.bmi_levelbar.set_unitsystem(Unitsystem::Metric);
                    self_.height_actionrow
                    .set_title(Some(&i18n("Height in centimeters")));
                    self_.weightgoal_actionrow
                        .set_title(Some(&i18n("Weightgoal in KG")));
                    self_.height_spin_button
                        .set_value(f64::from(Length::new::<inch>(get_spinbutton_value(&self_.height_spin_button)).get::<centimeter>()));
                    self_.weightgoal_spin_button
                        .set_value(f64::from(Mass::new::<pound>(get_spinbutton_value(&self_.height_spin_button)).get::<kilogram>()));
                } else {
                    self_.settings.set_unitsystem(Unitsystem::Imperial);
                    self_.bmi_levelbar.set_unitsystem(Unitsystem::Imperial);
                    self_.height_actionrow
                    .set_title(Some(&i18n("Height in inch")));
                    self_.weightgoal_actionrow
                        .set_title(Some(&i18n("Weightgoal in pounds")));
                    self_.height_spin_button
                        .set_value(f64::from(Length::new::<centimeter>(get_spinbutton_value(&self_.height_spin_button)).get::<inch>()));
                    self_.weightgoal_spin_button
                        .set_value(f64::from(Mass::new::<kilogram>(get_spinbutton_value(&self_.height_spin_button)).get::<pound>()));
                }
            }));

            self.export_activity_csv_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let file_chooser = gtk::FileChooserNativeBuilder::new()
                        .title(&i18n("Save Activities"))
                        .accept_label(&i18n("_Save"))
                        .cancel_label(&i18n("_Cancel"))
                        .modal(true)
                        .transient_for(&obj)
                        .action(gtk::FileChooserAction::Save)
                        .build();
                    file_chooser.set_current_name(&i18n("Activities.csv"));
                    file_chooser.connect_response(clone!(@weak obj, @strong file_chooser => move |_, r| {
                        if r == gtk::ResponseType::Accept {
                            let file = file_chooser.get_file().unwrap();
                            spawn!(async move {
                                let self_ = PreferencesWindow::from_instance(&obj);
                                let handler = CSVHandler::new(self_.db.get().unwrap().clone());
                                if let Err(e) = handler.export_activities_csv(&file).await {
                                    g_warning!(crate::config::LOG_DOMAIN, "{}", e.to_string());
                                }
                            });
                        }
                    }));
                    file_chooser.show();
                }));

            self.export_weight_csv_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let file_chooser = gtk::FileChooserNativeBuilder::new()
                        .title(&i18n("Save Weight Measurement"))
                        .accept_label(&i18n("_Save"))
                        .cancel_label(&i18n("_Cancel"))
                        .modal(true)
                        .transient_for(&obj)
                        .action(gtk::FileChooserAction::Save)
                        .build();
                    file_chooser.set_transient_for(Some(&obj));
                    file_chooser.set_current_name(&i18n("Weight Measurements.csv"));
                    file_chooser.connect_response(clone!(@weak obj, @strong file_chooser => move |_, r| {
                        if r == gtk::ResponseType::Accept {
                            let file = file_chooser.get_file().unwrap();
                            spawn!(async move {
                                let self_ = PreferencesWindow::from_instance(&obj);
                                let handler = CSVHandler::new(self_.db.get().unwrap().clone());
                                if let Err(e) = handler.export_weights_csv(&file).await {
                                    g_warning!(crate::config::LOG_DOMAIN, "{}", e.to_string());
                                }
                            });
                        }
                    }));
                    file_chooser.show();
                }));

            self.import_weight_csv_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let file_chooser = gtk::FileChooserNativeBuilder::new()
                        .title(&i18n("Open Weight Measurement"))
                        .accept_label(&i18n("_Open"))
                        .cancel_label(&i18n("_Cancel"))
                        .modal(true)
                        .transient_for(&obj)
                        .action(gtk::FileChooserAction::Open)
                        .build();
                    file_chooser.set_transient_for(Some(&obj));
                    file_chooser.connect_response(clone!(@weak obj, @strong file_chooser => move |_, r| {
                        if r == gtk::ResponseType::Accept {
                            let file = file_chooser.get_file().unwrap();
                            spawn!(async move {
                                let self_ = PreferencesWindow::from_instance(&obj);
                                let handler = CSVHandler::new(self_.db.get().unwrap().clone());
                                if let Err(e) = handler.import_weights_csv(&file).await {
                                    g_warning!(crate::config::LOG_DOMAIN, "{}", e.to_string());
                                }
                            });
                        }
                    }));
                    file_chooser.show();
                }));

            self.import_activity_csv_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let file_chooser = gtk::FileChooserNativeBuilder::new()
                        .title(&i18n("Open Activities"))
                        .accept_label(&i18n("_Open"))
                        .cancel_label(&i18n("_Cancel"))
                        .modal(true)
                        .transient_for(&obj)
                        .action(gtk::FileChooserAction::Open)
                        .build();
                    file_chooser.connect_response(clone!(@weak obj, @strong file_chooser => move |_, r| {
                        if r == gtk::ResponseType::Accept {
                            let file = file_chooser.get_file().unwrap();
                            spawn!(async move {
                                let self_ = PreferencesWindow::from_instance(&obj);
                                let handler = CSVHandler::new(self_.db.get().unwrap().clone());
                                if let Err(e) = handler.import_activities_csv(&file).await {
                                    g_warning!(crate::config::LOG_DOMAIN, "{}", e.to_string());
                                }
                            });
                        }
                    }));
                    file_chooser.show();
                }));
        }
    }
}

glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::PreferencesWindow;
}

impl PreferencesWindow {
    pub fn new(db: Database, parent_window: Option<gtk::Window>) -> Self {
        let o: PreferencesWindow =
            glib::Object::new(&[]).expect("Failed to create PreferencesWindow");

        o.set_transient_for(parent_window.as_ref());

        let self_ = imp::PreferencesWindow::from_instance(&o);
        self_.db.set(db.clone()).unwrap();
        self_.sync_list_box.set_database(db);
        self_.parent_window.set(parent_window).unwrap();

        o
    }

    pub fn connect_import_done<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("import-done", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }
}
