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
    core::{i18n, UnitSystem},
    model::NotificationFrequency,
    prelude::*,
    windows::{ExportDialog, ImportDialog},
};
use adw::prelude::*;
use chrono::{NaiveTime, Timelike};
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*},
};
use gtk_macros::{spawn, stateful_action};
use std::str::FromStr;
use uom::si::{
    f32::{Length, Mass},
    length::{centimeter, inch},
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::{Settings, UnitSystem},
        model::NotificationFrequency,
        widgets::{BmiLevelBar, DateSelector, SyncListBox, UnitSpinButton},
    };
    use adw::prelude::*;
    use gtk::{glib, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::{cell::Cell, str::FromStr};
    use uom::si::{
        f32::Mass,
        length::{centimeter, inch},
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        pub current_unit_system: Cell<UnitSystem>,
        pub settings: Settings,
        pub window_indentifier: OnceCell<ashpd::WindowIdentifier>,

        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_goal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub birthday_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub height_spin_button: TemplateChild<UnitSpinButton>,
        #[template_child]
        pub step_goal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub weight_goal_spin_button: TemplateChild<UnitSpinButton>,
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
        #[template_child]
        pub enable_notify: TemplateChild<gtk::Switch>,
        #[template_child]
        pub periodic_frequency_select: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub reminder_time: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub reminder_hour: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub reminder_minutes: TemplateChild<gtk::SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "HealthPreferencesWindow";
        type ParentType = adw::PreferencesWindow;
        type Type = super::PreferencesWindow;

        fn new() -> Self {
            let settings = Settings::instance();
            Self {
                current_unit_system: Cell::new(settings.unit_system()),
                settings,
                window_indentifier: OnceCell::new(),
                height_actionrow: TemplateChild::default(),
                weight_goal_actionrow: TemplateChild::default(),
                birthday_selector: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                step_goal_spin_button: TemplateChild::default(),
                weight_goal_spin_button: TemplateChild::default(),
                bmi_levelbar: TemplateChild::default(),
                sync_list_box: TemplateChild::default(),
                export_csv_button: TemplateChild::default(),
                import_csv_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                enable_notify: TemplateChild::default(),
                reminder_time: TemplateChild::default(),
                periodic_frequency_select: TemplateChild::default(),
                reminder_hour: TemplateChild::default(),
                reminder_minutes: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            UnitSpinButton::static_type();
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.current_unit_system.get() == UnitSystem::Metric {
                self.height_spin_button
                    .set_value(f64::from(self.settings.user_height().get::<centimeter>()));
                self.weight_goal_spin_button.set_value(f64::from(
                    self.settings
                        .user_weight_goal()
                        .unwrap_or_else(|| Mass::new::<kilogram>(-1.0))
                        .get::<kilogram>(),
                ));
            } else {
                self.height_spin_button
                    .set_value(f64::from(self.settings.user_height().get::<inch>()));
                self.weight_goal_spin_button.set_value(f64::from(
                    self.settings
                        .user_weight_goal()
                        .unwrap_or_else(|| Mass::new::<pound>(-1.0))
                        .get::<pound>(),
                ));
            }

            self.step_goal_spin_button
                .set_value(f64::from(self.settings.user_step_goal()));
            if let Some(date) = self.settings.user_birthday() {
                self.birthday_selector
                    .set_selected_date(date.and_hms(0, 0, 0));
            }
            self.bmi_levelbar.set_height(self.settings.user_height());

            self.bmi_levelbar.set_weight(
                self.settings
                    .user_weight_goal()
                    .unwrap_or_else(|| Mass::new::<kilogram>(1.0)),
            );
            obj.setup_actions();
            obj.connect_handlers();
            obj.handle_enable_notify_changed(true);
            obj.init_time_buttons();
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "notification-frequency",
                    "notification-frequency",
                    "notification-frequency",
                    Some("every_4_hrs"),
                    glib::ParamFlags::WRITABLE,
                )]
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
                "notification-frequency" => {
                    let frequency =
                        NotificationFrequency::from_str(&value.get::<String>().unwrap()).unwrap();
                    self.settings.set_notification_frequency(frequency);
                    self.reminder_time
                        .set_visible(frequency == NotificationFrequency::Fixed);
                }
                _ => unimplemented!(),
            }
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
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl PreferencesWindow {
    pub fn connect_import_done<F: Fn(&Self) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("import-done", false, move |values| {
            callback(&values[0].get().unwrap());
            None
        })
    }

    /// Create a new [PreferencesWindow].
    ///
    /// # Arguments
    /// * `parent_window` - The transient parent of the window.
    ///
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        glib::Object::new(&[
            ("transient-for", &parent_window.as_ref()),
            (
                "application",
                &parent_window
                    .as_ref()
                    .and_then(gtk::prelude::GtkWindowExt::application)
                    .as_ref(),
            ),
        ])
        .expect("Failed to create PreferencesWindow")
    }

    fn handle_frequency(&self, action: &gio::SimpleAction, parameter: Option<&glib::Variant>) {
        let parameter = parameter.unwrap();
        self.set_notification_frequency(
            NotificationFrequency::from_str(parameter.get::<String>().unwrap().as_str()).unwrap(),
        );
        action.set_state(parameter);
    }

    fn setup_actions(&self) {
        let action_group = gtk::gio::SimpleActionGroup::new();

        stateful_action!(
            action_group,
            "frequency",
            Some(&String::static_variant_type()),
            "hourly",
            clone!(@weak self as obj => move |a, p| {
                obj.handle_frequency(a, p);
            })
        );

        self.insert_action_group("notification", Some(&action_group));
    }

    fn set_notification_frequency(&self, frequency: NotificationFrequency) {
        self.set_property("notification-frequency", frequency)
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_
            .settings
            .connect_unit_system_changed(clone!(@weak self as obj => move |_, _| {
                obj.handle_unit_system_changed();
            }));
    }

    fn imp(&self) -> &imp::PreferencesWindow {
        imp::PreferencesWindow::from_instance(self)
    }

    #[template_callback]
    fn handle_birthday_selector_changed(&self) {
        let self_ = self.imp();
        self_
            .settings
            .set_user_birthday(self_.birthday_selector.selected_date().date());
    }

    fn handle_enable_notify_changed(&self, initializing: bool) {
        let self_ = self.imp();
        let switch_state = if initializing {
            self_.settings.enable_notifications()
        } else {
            self_.enable_notify.is_active()
        };
        println!("Enabling not: {}", switch_state);
        self_.settings.set_enable_notifications(switch_state);
        self_.enable_notify.set_active(switch_state);
        self_.periodic_frequency_select.set_visible(switch_state);
        self_.reminder_time.set_visible(
            self_.settings.enable_notifications()
                && self_.settings.notification_frequency() == NotificationFrequency::Fixed,
        );
        if switch_state && ashpd::is_sandboxed() && !initializing {
            spawn!(clone!(@weak self as obj => async move {
                obj.handle_sandbox_autostart().await;
            }));
        }
    }

    #[template_callback]
    fn handle_enable_notify_changed_callback(&self) {
        self.handle_enable_notify_changed(false);
    }

    #[template_callback]
    fn handle_export_csv_button_clicked(&self) {
        let dialog = ExportDialog::new(self.transient_for().as_ref());
        dialog.show();
    }

    #[template_callback]
    fn handle_height_spin_button_changed(&self) {
        let self_ = self.imp();
        if let Some(val) = self_.height_spin_button.raw_value::<f32>() {
            let height = if self_.current_unit_system.get() == UnitSystem::Metric {
                Length::new::<centimeter>(val)
            } else {
                Length::new::<inch>(val)
            };

            self_.settings.set_user_height(height);
            self_.bmi_levelbar.set_height(height);
        }
    }

    #[template_callback]
    fn handle_import_csv_button_clicked(&self) {
        let dialog = ImportDialog::new(self.transient_for().as_ref());
        dialog.show();
    }

    async fn handle_sandbox_autostart(&self) {
        let self_ = self.imp();
        let window_indentifier = if let Some(i) = self_.window_indentifier.get() {
            i
        } else {
            let i =
                ashpd::WindowIdentifier::from_native(self.upcast_ref::<adw::PreferencesWindow>())
                    .await;
            self_.window_indentifier.set(i).unwrap();
            self_.window_indentifier.get().unwrap()
        };
        match ashpd::desktop::background::request(
            window_indentifier,
            &i18n("Remind you of your step goals"),
            true,
            Some(&[crate::config::APPLICATION_ID, "--gapplication-service"]),
            false,
        )
        .await
        {
            Ok(r) => {
                if !r.auto_start() {
                    glib::g_warning!(
                        crate::config::LOG_DOMAIN,
                        "Permission to be autostarted was denied..."
                    )
                }
            }
            Err(e) => glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Couldn't request to stay active in background: {}",
                e.to_string()
            ),
        }
    }

    #[template_callback]
    fn handle_step_goal_spin_button_changed(&self) {
        let self_ = self.imp();
        if let Some(val) = self_.step_goal_spin_button.raw_value::<u32>() {
            self_.settings.set_user_step_goal(val);
        }
    }

    fn handle_unit_system_changed(&self) {
        let self_ = self.imp();
        let unit_system = self_.settings.unit_system();

        if unit_system == UnitSystem::Imperial && !self_.unit_imperial_togglebutton.is_active() {
            self_.unit_imperial_togglebutton.set_active(true);
        } else if unit_system == UnitSystem::Metric && !self_.unit_metric_togglebutton.is_active() {
            self_.unit_metric_togglebutton.set_active(true);
        }

        if self_.current_unit_system.get() == unit_system {
            return;
        }

        self_.current_unit_system.set(unit_system);

        if unit_system == UnitSystem::Metric {
            self_.height_spin_button.set_value(f64::from(
                Length::new::<inch>(self_.height_spin_button.raw_value().unwrap_or_default())
                    .get::<centimeter>(),
            ));
            self_.weight_goal_spin_button.set_value(f64::from(
                Mass::new::<pound>(
                    self_
                        .weight_goal_spin_button
                        .raw_value()
                        .unwrap_or_default(),
                )
                .get::<kilogram>(),
            ));
        } else {
            self_.height_spin_button.set_value(f64::from(
                Length::new::<centimeter>(self_.height_spin_button.raw_value().unwrap_or_default())
                    .get::<inch>(),
            ));
            self_.weight_goal_spin_button.set_value(f64::from(
                Mass::new::<kilogram>(
                    self_
                        .weight_goal_spin_button
                        .raw_value()
                        .unwrap_or_default(),
                )
                .get::<pound>(),
            ));
        }
    }

    #[template_callback]
    fn handle_weight_goal_spin_button_changed(&self) {
        let self_ = self.imp();
        if let Some(val) = self_.weight_goal_spin_button.raw_value::<f32>() {
            let weight = if self_.current_unit_system.get() == UnitSystem::Metric {
                Mass::new::<kilogram>(val)
            } else {
                Mass::new::<pound>(val)
            };

            self_.settings.set_user_weight_goal(weight);
            self_.bmi_levelbar.set_weight(weight);
        }
    }

    fn init_time_buttons(&self) {
        let self_ = self.imp();
        let notify_time = self_.settings.notification_time();
        self_.reminder_hour.set_value(f64::from(notify_time.hour()));
        self_
            .reminder_minutes
            .set_value(f64::from(notify_time.minute()));
    }

    #[template_callback]
    fn handle_close_window(&self) -> bool {
        let self_ = self.imp();
        let remind_time = NaiveTime::from_hms_milli(
            self_.reminder_hour.value_as_int() as u32,
            self_.reminder_minutes.value_as_int() as u32,
            0,
            0,
        );
        self_.settings.set_notification_time(remind_time);
        false
    }
}

#[cfg(test)]
mod test {
    use super::PreferencesWindow;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();

        PreferencesWindow::new(None);
    }
}
