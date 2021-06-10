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

use crate::{
    core::{i18n, settings::prelude::*, utils::spinbutton_value, Unitsystem},
    windows::{ExportDialog, ImportDialog},
};
use adw::prelude::*;
use glib::{clone, subclass::prelude::*};
use gtk::prelude::*;
use uom::si::{
    f32::{Length, Mass},
    length::{centimeter, inch},
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::{i18n, settings::prelude::*, Unitsystem},
        widgets::{BmiLevelBar, DateSelector, SyncListBox},
    };
    use adw::prelude::*;
    use gio::Settings;
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;
    use uom::si::{
        length::{centimeter, inch},
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        pub current_unitsystem: Cell<Unitsystem>,
        pub parent_window: OnceCell<Option<gtk::Window>>,
        pub settings: Settings,

        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weightgoal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub birthday_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub height_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub stepgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub weightgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub bmi_levelbar: TemplateChild<BmiLevelBar>,
        #[template_child]
        pub sync_list_box: TemplateChild<SyncListBox>,
        #[template_child]
        pub export_csv_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub import_csv_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub unit_imperial_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub unit_metric_togglebutton: TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "HealthPreferencesWindow";
        type ParentType = adw::PreferencesWindow;
        type Type = super::PreferencesWindow;

        fn new() -> Self {
            let settings = Settings::instance();
            Self {
                current_unitsystem: Cell::new(settings.unitsystem()),
                settings,
                height_actionrow: TemplateChild::default(),
                weightgoal_actionrow: TemplateChild::default(),
                birthday_selector: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                stepgoal_spin_button: TemplateChild::default(),
                weightgoal_spin_button: TemplateChild::default(),
                bmi_levelbar: TemplateChild::default(),
                parent_window: OnceCell::new(),
                sync_list_box: TemplateChild::default(),
                export_csv_button: TemplateChild::default(),
                import_csv_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.current_unitsystem.get() == Unitsystem::Metric {
                self.height_actionrow
                    .set_title(Some(&i18n("Height in centimeters")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in KG")));
                self.height_spin_button
                    .set_value(f64::from(self.settings.user_height().get::<centimeter>()));
                self.weightgoal_spin_button
                    .set_value(f64::from(self.settings.user_weightgoal().get::<kilogram>()));
            } else {
                self.height_actionrow
                    .set_title(Some(&i18n("Height in inch")));
                self.weightgoal_actionrow
                    .set_title(Some(&i18n("Weightgoal in pounds")));
                self.height_spin_button
                    .set_value(f64::from(self.settings.user_height().get::<inch>()));
                self.weightgoal_spin_button
                    .set_value(f64::from(self.settings.user_weightgoal().get::<pound>()));
            }

            self.stepgoal_spin_button
                .set_value(f64::from(self.settings.user_stepgoal()));
            if let Some(date) = self.settings.user_birthday() {
                self.birthday_selector
                    .set_selected_date(date.and_hms(0, 0, 0));
            }
            self.bmi_levelbar.set_height(self.settings.user_height());

            self.bmi_levelbar
                .set_weight(self.settings.user_weightgoal());
            obj.connect_handlers();
        }
    }

    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl adw::subclass::window::AdwWindowImpl for PreferencesWindow {}
    impl adw::subclass::preferences_window::PreferencesWindowImpl for PreferencesWindow {}
}

glib::wrapper! {
    /// The [PreferencesWindow] is presented to the user to set certain settings
    /// in the application.
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::PreferencesWindow;
}

impl PreferencesWindow {
    pub fn connect_import_done<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("import-done", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    /// Create a new [PreferencesWindow].
    ///
    /// # Arguments
    /// * `parent_window` - The transient parent of the window.
    ///
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create PreferencesWindow");

        o.set_transient_for(parent_window.as_ref());
        o.set_application(
            parent_window
                .as_ref()
                .and_then(|p| p.application())
                .as_ref(),
        );

        let self_ = o.imp();
        self_.parent_window.set(parent_window).unwrap();

        o
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_
            .settings
            .connect_unitsystem_changed(clone!(@weak self as obj => move |_, _| {
                obj.handle_unitsystem_changed();
            }));

        self_.birthday_selector.connect_selected_date_notify(
            clone!(@weak self as obj => move |_| {
                obj.handle_birthday_selector_changed();
            }),
        );

        self_
            .export_csv_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_export_csv_button_clicked();
            }));

        self_
            .height_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_height_spin_button_changed();
            }));

        self_
            .import_csv_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_import_csv_button_clicked();
            }));

        self_
            .stepgoal_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_stepgoal_spin_button_changed();
            }));

        self_
            .weightgoal_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_weightgoal_spin_button_changed();
            }));
    }

    fn imp(&self) -> &imp::PreferencesWindow {
        imp::PreferencesWindow::from_instance(self)
    }

    fn handle_birthday_selector_changed(&self) {
        let self_ = self.imp();
        self_
            .settings
            .set_user_birthday(self_.birthday_selector.selected_date().date());
    }

    fn handle_export_csv_button_clicked(&self) {
        let dialog = ExportDialog::new(self.imp().parent_window.get().unwrap().as_ref());
        dialog.show();
    }

    fn handle_height_spin_button_changed(&self) {
        let self_ = self.imp();
        let val = spinbutton_value::<u32>(&self_.height_spin_button) as f32;
        if val != 0.0 {
            let height = if self_.current_unitsystem.get() == Unitsystem::Metric {
                Length::new::<centimeter>(val)
            } else {
                Length::new::<inch>(val)
            };

            self_.settings.set_user_height(height);
            self_.bmi_levelbar.set_height(height);
        }
    }

    fn handle_import_csv_button_clicked(&self) {
        let dialog = ImportDialog::new(self.imp().parent_window.get().unwrap().as_ref());
        dialog.show();
    }

    fn handle_stepgoal_spin_button_changed(&self) {
        let self_ = self.imp();
        let val = spinbutton_value::<u32>(&self_.stepgoal_spin_button);
        if val != 0 {
            self_.settings.set_user_stepgoal(val);
        }
    }

    fn handle_unitsystem_changed(&self) {
        let self_ = self.imp();
        let unitsystem = self_.settings.unitsystem();

        if unitsystem == Unitsystem::Imperial && !self_.unit_imperial_togglebutton.is_active() {
            self_.unit_imperial_togglebutton.set_active(true);
        } else if unitsystem == Unitsystem::Metric && !self_.unit_metric_togglebutton.is_active() {
            self_.unit_metric_togglebutton.set_active(true);
        }

        if self_.current_unitsystem.get() == unitsystem {
            return;
        }

        self_.current_unitsystem.set(unitsystem);

        if unitsystem == Unitsystem::Metric {
            self_
                .height_actionrow
                .set_title(Some(&i18n("Height in centimeters")));
            self_
                .weightgoal_actionrow
                .set_title(Some(&i18n("Weightgoal in KG")));
            self_.height_spin_button.set_value(f64::from(
                Length::new::<inch>(spinbutton_value(&self_.height_spin_button))
                    .get::<centimeter>(),
            ));
            self_.weightgoal_spin_button.set_value(f64::from(
                Mass::new::<pound>(spinbutton_value(&self_.weightgoal_spin_button))
                    .get::<kilogram>(),
            ));
        } else {
            self_
                .height_actionrow
                .set_title(Some(&i18n("Height in inch")));
            self_
                .weightgoal_actionrow
                .set_title(Some(&i18n("Weightgoal in pounds")));
            self_.height_spin_button.set_value(f64::from(
                Length::new::<centimeter>(spinbutton_value(&self_.height_spin_button))
                    .get::<inch>(),
            ));
            self_.weightgoal_spin_button.set_value(f64::from(
                Mass::new::<kilogram>(spinbutton_value(&self_.weightgoal_spin_button))
                    .get::<pound>(),
            ));
        }
    }

    fn handle_weightgoal_spin_button_changed(&self) {
        let self_ = self.imp();
        let val = spinbutton_value::<f32>(&self_.weightgoal_spin_button);
        if val != 0.0 {
            let weight = if self_.current_unitsystem.get() == Unitsystem::Metric {
                Mass::new::<kilogram>(val)
            } else {
                Mass::new::<pound>(val)
            };

            self_.settings.set_user_weightgoal(weight);
            self_.bmi_levelbar.set_weight(weight);
        }
    }
}
