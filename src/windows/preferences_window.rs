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
    core::UnitSystem,
    model::{NotificationFrequency, User},
    prelude::*,
    windows::{ExportDialog, ImportDialog},
};
use adw::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*},
};
use gtk_macros::stateful_action;
use std::str::FromStr;
use uom::si::{
    f32::{Length, Mass},
    length::{centimeter, inch},
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::{Database, Settings, UnitSystem},
        model::NotificationFrequency,
        widgets::{BmiLevelBar, DateSelector, SyncListBox, UnitSpinButton},
    };
    use adw::prelude::*;
    use gtk::{glib, subclass::prelude::*, CompositeTemplate};
    use std::{cell::Cell, str::FromStr};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        pub current_unit_system: Cell<UnitSystem>,
        pub settings: Settings,
        pub database: Database,

        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_goal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub user_name_entry: TemplateChild<gtk::Entry>,
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
            let database = Database::instance();
            Self {
                current_unit_system: Cell::new(settings.unit_system()),
                settings,
                database,
                height_actionrow: TemplateChild::default(),
                weight_goal_actionrow: TemplateChild::default(),
                user_name_entry: TemplateChild::default(),
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
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            gtk_macros::spawn!(glib::clone!(@weak obj => async move {
                obj.construct_user().await;
                obj.setup_actions();
                obj.connect_handlers();
                obj.handle_enable_notify_changed(true);
                obj.init_time_buttons();
            }));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::builder("notification-frequency")
                    .default_value(Some("every_4_hrs"))
                    .write_only()
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "notification-frequency" => {
                    let frequency =
                        NotificationFrequency::from_str(value.get::<&str>().unwrap()).unwrap();
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
            self.imp().settings.notification_frequency(),
            clone!(@weak self as obj => move |a, p| {
                obj.handle_frequency(a, p);
            })
        );

        self.insert_action_group("notification", Some(&action_group));
    }

    pub async fn get_user(&self) -> User {
        let imp = self.imp();
        let user_id = i64::from(imp.settings.active_user_id());
        let user = &imp.database.user(user_id).await.unwrap();
        user.clone()
    }

    pub async fn construct_user(&self) {
        let imp = self.imp();
        let user = self.get_user().await;
        if imp.current_unit_system.get() == UnitSystem::Metric {
            imp.height_spin_button.set_value(f64::from(
                user.user_height()
                    .unwrap_or_else(|| Length::new::<centimeter>(-1.0))
                    .get::<centimeter>(),
            ));
            imp.weight_goal_spin_button.set_value(f64::from(
                user.user_weightgoal()
                    .unwrap_or_else(|| Mass::new::<kilogram>(-1.0))
                    .get::<kilogram>(),
            ));
        } else {
            imp.height_spin_button.set_value(f64::from(
                user.user_height()
                    .unwrap_or_else(|| Length::new::<inch>(-1.0))
                    .get::<inch>(),
            ));
            imp.weight_goal_spin_button.set_value(f64::from(
                user.user_weightgoal()
                    .unwrap_or_else(|| Mass::new::<pound>(-1.0))
                    .get::<pound>(),
            ));
        }
        imp.user_name_entry
            .set_text(user.user_name().unwrap_or_default().as_str());
        imp.step_goal_spin_button
            .set_value(f64::from(user.user_stepgoal().unwrap() as i32));
        if let Some(date) = user.user_birthday() {
            imp.birthday_selector.set_selected_date(date);
        }
        imp.bmi_levelbar.set_height(
            user.user_height()
                .unwrap_or_else(|| Length::new::<centimeter>(1.0)),
        );

        imp.bmi_levelbar.set_weight(
            user.user_weightgoal()
                .unwrap_or_else(|| Mass::new::<kilogram>(1.0)),
        );
    }

    fn set_notification_frequency(&self, frequency: NotificationFrequency) {
        self.set_property("notification-frequency", frequency)
    }

    fn connect_handlers(&self) {
        let imp = self.imp();

        imp.settings
            .connect_unit_system_changed(clone!(@weak self as obj => move |_, _| {
                obj.handle_unit_system_changed();
            }));
        imp.height_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_height_spin_button_changed();
            }));
        imp.weight_goal_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_weight_goal_spin_button_changed();
            }));
        imp.step_goal_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_step_goal_spin_button_changed();
            }));
    }

    pub async fn update_user(&self, user: User) {
        let imp = self.imp();
        if let Err(e) = imp.database.update_user(user).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to update the user data due to error {e}",
            )
        }
    }

    #[template_callback]
    fn handle_user_name_entry_changed(&self) {
        let imp = self.imp();
        let user_name = imp.user_name_entry.text().to_string();
        glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
            let user = obj.get_user().await;
            user.set_user_name(Some(user_name.as_str()));
            obj.update_user(user).await;
        }));
    }

    #[template_callback]
    fn handle_birthday_selector_changed(&self) {
        let imp = self.imp();
        let user_birthday = imp.birthday_selector.selected_date();
        glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
            let user = obj.get_user().await;
            user.set_user_birthday(Some(user_birthday));
            obj.update_user(user).await;
        }));
    }

    fn handle_enable_notify_changed(&self, initializing: bool) {
        let imp = self.imp();
        let switch_state = if initializing {
            imp.settings.enable_notifications()
        } else {
            imp.enable_notify.is_active()
        };
        imp.settings.set_enable_notifications(switch_state);
        imp.enable_notify.set_active(switch_state);
        imp.periodic_frequency_select.set_visible(switch_state);
        imp.reminder_time.set_visible(
            imp.settings.enable_notifications()
                && imp.settings.notification_frequency() == NotificationFrequency::Fixed,
        );
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

    fn handle_height_spin_button_changed(&self) {
        let imp = self.imp();
        if let Some(val) = imp.height_spin_button.raw_value::<f32>() {
            let height = if imp.current_unit_system.get() == UnitSystem::Metric {
                Length::new::<centimeter>(val)
            } else {
                Length::new::<inch>(val)
            };

            glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
                let user = obj.get_user().await;
                user.set_user_height(Some(height));
                obj.update_user(user).await;
            }));
            imp.bmi_levelbar.set_height(height);
        }
    }

    #[template_callback]
    fn handle_import_csv_button_clicked(&self) {
        let dialog = ImportDialog::new(self.transient_for().as_ref());
        dialog.show();
    }

    fn handle_step_goal_spin_button_changed(&self) {
        let imp = self.imp();
        if let Some(val) = imp.step_goal_spin_button.raw_value::<u32>() {
            glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
                let user = obj.get_user().await;
                user.set_user_stepgoal(Some(i64::from(val)));
                obj.update_user(user).await;
            }));
        }
    }

    fn handle_unit_system_changed(&self) {
        let imp = self.imp();
        let unit_system = imp.settings.unit_system();

        if unit_system == UnitSystem::Imperial && !imp.unit_imperial_togglebutton.is_active() {
            imp.unit_imperial_togglebutton.set_active(true);
        } else if unit_system == UnitSystem::Metric && !imp.unit_metric_togglebutton.is_active() {
            imp.unit_metric_togglebutton.set_active(true);
        }

        if imp.current_unit_system.get() == unit_system {
            return;
        }

        imp.current_unit_system.set(unit_system);

        if unit_system == UnitSystem::Metric {
            imp.height_spin_button.set_value(f64::from(
                Length::new::<inch>(imp.height_spin_button.raw_value().unwrap_or_default())
                    .get::<centimeter>(),
            ));
            imp.weight_goal_spin_button.set_value(f64::from(
                Mass::new::<pound>(imp.weight_goal_spin_button.raw_value().unwrap_or_default())
                    .get::<kilogram>(),
            ));
        } else {
            imp.height_spin_button.set_value(f64::from(
                Length::new::<centimeter>(imp.height_spin_button.raw_value().unwrap_or_default())
                    .get::<inch>(),
            ));
            imp.weight_goal_spin_button.set_value(f64::from(
                Mass::new::<kilogram>(imp.weight_goal_spin_button.raw_value().unwrap_or_default())
                    .get::<pound>(),
            ));
        }
    }

    fn handle_weight_goal_spin_button_changed(&self) {
        let imp = self.imp();
        if let Some(val) = imp.weight_goal_spin_button.raw_value::<f32>() {
            let weight = if imp.current_unit_system.get() == UnitSystem::Metric {
                Mass::new::<kilogram>(val)
            } else {
                Mass::new::<pound>(val)
            };

            glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
                let user = obj.get_user().await;
                user.set_user_weightgoal(Some(weight));
                obj.update_user(user).await;
            }));
            imp.bmi_levelbar.set_weight(weight);
        }
    }

    fn init_time_buttons(&self) {
        let imp = self.imp();
        let notify_time = imp.settings.notification_time();
        imp.reminder_hour.set_value(f64::from(notify_time.hour()));
        imp.reminder_minutes
            .set_value(f64::from(notify_time.minutes()));
    }

    #[template_callback]
    fn handle_close_window(&self) -> bool {
        let imp = self.imp();
        let remind_time = Time::new(
            imp.reminder_hour.value_as_int().try_into().unwrap(),
            imp.reminder_minutes.value_as_int().try_into().unwrap(),
            0,
        )
        .unwrap();
        imp.settings.set_notification_time(remind_time);
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
