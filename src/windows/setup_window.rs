/* setup_window.rs
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
    model::{User, Weight},
    plugins::PluginName,
    prelude::*,
};
use adw::prelude::*;
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*},
};
use gtk_macros::action;
use uom::si::{
    f32::{Length, Mass},
    length::{centimeter, inch, meter},
    mass::{kilogram, pound},
};

static OPTIMAL_BMI: f32 = 22.5;

mod imp {
    use crate::{
        core::{Database, Settings, UnitSystem},
        widgets::{BmiLevelBar, DateSelector, SyncListBox, UnitSpinButton},
    };
    use gtk::{
        glib::{self, subclass::Signal},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::Cell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/setup_window.ui")]
    pub struct SetupWindow {
        pub current_unit_system: Cell<UnitSystem>,
        pub settings: Settings,
        pub database: Database,

        #[template_child]
        pub current_bmi_levelbar: TemplateChild<BmiLevelBar>,
        #[template_child]
        pub target_bmi_levelbar: TemplateChild<BmiLevelBar>,
        #[template_child]
        pub setup_first_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub setup_second_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub setup_third_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub setup_fourth_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub setup_done_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub setup_quit_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub setup_next_page_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub setup_previous_page_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub setup_right_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub setup_left_stack: TemplateChild<gtk::Stack>,
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
        pub weight_spin_button: TemplateChild<UnitSpinButton>,
        #[template_child]
        pub unit_imperial_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub unit_metric_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_goal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub setup_carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub sync_list_box: TemplateChild<SyncListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SetupWindow {
        const NAME: &'static str = "HealthSetupWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::SetupWindow;

        fn new() -> Self {
            let settings = Settings::instance();
            let database = Database::instance();
            Self {
                current_unit_system: Cell::new(settings.unit_system()),
                settings,
                database,
                current_bmi_levelbar: TemplateChild::default(),
                target_bmi_levelbar: TemplateChild::default(),
                setup_first_page: TemplateChild::default(),
                setup_second_page: TemplateChild::default(),
                setup_third_page: TemplateChild::default(),
                setup_fourth_page: TemplateChild::default(),
                setup_done_button: TemplateChild::default(),
                setup_quit_button: TemplateChild::default(),
                setup_next_page_button: TemplateChild::default(),
                setup_previous_page_button: TemplateChild::default(),
                setup_right_stack: TemplateChild::default(),
                setup_left_stack: TemplateChild::default(),
                user_name_entry: TemplateChild::default(),
                birthday_selector: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                step_goal_spin_button: TemplateChild::default(),
                weight_goal_spin_button: TemplateChild::default(),
                weight_spin_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                height_actionrow: TemplateChild::default(),
                weight_goal_actionrow: TemplateChild::default(),
                weight_actionrow: TemplateChild::default(),
                setup_carousel: TemplateChild::default(),
                sync_list_box: TemplateChild::default(),
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

    impl ObjectImpl for SetupWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.step_goal_spin_button.set_value(10000.0);

            obj.connect_handlers();
            obj.setup_actions();
            obj.setup_unit_system_text(self.settings.unit_system());
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("setup-done", &[], glib::Type::UNIT.into()).build()]
            });

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SetupWindow {}
    impl WindowImpl for SetupWindow {}
    impl ApplicationWindowImpl for SetupWindow {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for SetupWindow {}
}

glib::wrapper! {
  /// The [SetupWindow] is shown to the user on the first start of the applcation to fill in some data.
  pub struct SetupWindow(ObjectSubclass<imp::SetupWindow>)
      @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
      @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl SetupWindow {
    /// Connect to the setup being completed by the user.
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_setup_done<F: Fn(&Self) + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("setup-done", false, move |values| {
            callback(&values[0].get().unwrap());
            None
        })
    }

    /// Create a new [SetupWindow].
    ///
    /// # Arguments
    /// * `app` - The [GtkApplication](gtk::Application) to use.
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create SetupWindow")
    }

    fn connect_handlers(&self) {
        let imp = self.imp();

        imp.settings
            .connect_unit_system_changed(clone!(@weak self as obj => move |_, _| {
                obj.handle_unit_system_changed();
            }));
    }
    fn handle_fullscreen(&self) {
        if self.is_fullscreen() {
            self.unfullscreen();
        } else {
            self.fullscreen();
        }
    }

    #[template_callback]
    fn handle_height_spin_button_changed(&self) {
        let imp = self.imp();
        self.set_optimal_weight_goal();
        self.try_enable_next_button_first_page();

        let unitless_height = imp.height_spin_button.raw_value().unwrap_or_default();
        let height = if imp.current_unit_system.get() == UnitSystem::Metric {
            Length::new::<centimeter>(unitless_height)
        } else {
            Length::new::<inch>(unitless_height)
        };
        imp.current_bmi_levelbar.set_height(height);
        imp.target_bmi_levelbar.set_height(height);

        imp.current_bmi_levelbar.set_bmi_label(&i18n("Current BMI"));
        imp.target_bmi_levelbar.set_bmi_label(&i18n("Target BMI"));
    }

    #[template_callback]
    fn handle_setup_carousel_page_changed(&self, index: u32, carousel: adw::Carousel) {
        let imp = self.imp();

        if carousel.n_pages() - 1 == index {
            imp.setup_done_button.set_visible(true);
            imp.setup_right_stack
                .set_visible_child(&imp.setup_done_button.get());
        } else if index == 0 {
            imp.setup_quit_button.set_visible(true);
            imp.setup_left_stack
                .set_visible_child(&imp.setup_quit_button.get());
        } else {
            imp.setup_next_page_button.set_visible(true);
            imp.setup_previous_page_button.set_visible(true);
            imp.setup_right_stack
                .set_visible_child(&imp.setup_next_page_button.get());
            imp.setup_left_stack
                .set_visible_child(&imp.setup_previous_page_button.get());
        }
    }

    pub async fn handle_response(&self, id: gtk::ResponseType) {
        if id == gtk::ResponseType::Ok {
            let imp = self.imp();
            let top_unused_user_id = imp.database.get_top_unused_user_id().await.unwrap();

            let unitless_height = imp.height_spin_button.raw_value().unwrap_or_default();

            let height = if imp.current_unit_system.get() == UnitSystem::Metric {
                imp.settings.set_unit_system(UnitSystem::Metric);
                Length::new::<centimeter>(unitless_height)
            } else {
                imp.settings.set_unit_system(UnitSystem::Imperial);
                Length::new::<inch>(unitless_height)
            };

            let unitless_weight_goal = imp.weight_goal_spin_button.raw_value().unwrap_or_default();
            let weight_goal = if imp.current_unit_system.get() == UnitSystem::Metric {
                Mass::new::<kilogram>(unitless_weight_goal)
            } else {
                Mass::new::<pound>(unitless_weight_goal)
            };

            let mut user_builder = User::builder();
            user_builder
                .user_id(top_unused_user_id)
                .user_name(imp.user_name_entry.text().as_str())
                .user_birthday(imp.birthday_selector.selected_date())
                .user_height(height)
                .user_weightgoal(weight_goal)
                .user_stepgoal(imp.step_goal_spin_button.raw_value().unwrap_or_default())
                .enabled_plugins(vec![
                    PluginName::Activities,
                    PluginName::Calories,
                    PluginName::Weight,
                    PluginName::Steps,
                ])
                .recent_activity_types(vec![])
                .did_initial_setup(true);

            let user = user_builder.build();

            if let Err(e) = imp.database.create_user(user).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to save new data due to error {e}",
                )
            }
            imp.settings.set_active_user_id(top_unused_user_id as u32);
        }
    }

    pub async fn add_weight(&self) {
        let imp = self.imp();
        let unitless_weight = imp.weight_spin_button.raw_value().unwrap_or_default();
        let weight = if imp.current_unit_system.get() == UnitSystem::Metric {
            Mass::new::<kilogram>(unitless_weight)
        } else {
            Mass::new::<pound>(unitless_weight)
        };
        if let Err(e) = imp
            .database
            .save_weight(Weight::new(glib::DateTime::local(), weight))
            .await
        {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to save new data due to error {e}",
            )
        }
    }

    #[template_callback]
    fn handle_setup_done_button_clicked(&self) {
        glib::MainContext::default().spawn_local(clone!(@weak self as obj => async move {
            obj.handle_response(gtk::ResponseType::Ok).await;
            obj.add_weight().await;
            obj.emit_by_name::<()>("setup-done", &[]);
        }));

        self.destroy();
    }

    #[template_callback]
    fn handle_setup_next_page_button_clicked(&self) {
        let imp = self.imp();
        match imp.setup_carousel.position() as u32 {
            0 => imp
                .setup_carousel
                .scroll_to(&imp.setup_second_page.get(), true),
            1 => {
                if imp.setup_next_page_button.is_sensitive()
                    && imp.weight_spin_button.has_default_value()
                {
                    imp.setup_next_page_button.set_sensitive(false);
                }
                imp.setup_carousel
                    .scroll_to(&imp.setup_third_page.get(), true)
            }
            2 => imp
                .setup_carousel
                .scroll_to(&imp.setup_fourth_page.get(), true),
            3 => imp.setup_done_button.emit_clicked(),
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_setup_previous_page_button_clicked(&self) {
        let imp = self.imp();
        match imp.setup_carousel.position() as u32 {
            0 => self.destroy(),
            1 => imp
                .setup_carousel
                .scroll_to(&imp.setup_first_page.get(), true),
            2 => {
                if !imp.setup_next_page_button.is_sensitive() {
                    imp.setup_next_page_button.set_sensitive(true);
                }
                imp.setup_carousel
                    .scroll_to(&imp.setup_second_page.get(), true)
            }
            3 => imp
                .setup_carousel
                .scroll_to(&imp.setup_third_page.get(), true),
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_setup_quit_button_clicked(&self) {
        self.destroy();
    }

    #[template_callback]
    fn handle_weight_spin_button_changed(&self) {
        let imp = self.imp();
        self.try_enable_next_button_third_page();
        let unitless_weight = imp.weight_spin_button.raw_value().unwrap_or_default();
        let weight = if imp.current_unit_system.get() == UnitSystem::Metric {
            Mass::new::<kilogram>(unitless_weight)
        } else {
            Mass::new::<pound>(unitless_weight)
        };

        imp.current_bmi_levelbar.set_weight(weight);
    }

    #[template_callback]
    fn handle_weight_goal_spin_button_changed(&self) {
        let imp = self.imp();
        let unitless_weight = imp.weight_goal_spin_button.raw_value().unwrap_or_default();
        let weight = if imp.current_unit_system.get() == UnitSystem::Metric {
            Mass::new::<kilogram>(unitless_weight)
        } else {
            Mass::new::<pound>(unitless_weight)
        };

        imp.target_bmi_levelbar.set_weight(weight);
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

        self.setup_unit_system_text(unit_system);
    }
    fn setup_actions(&self) {
        action!(
            self,
            "quit",
            clone!(@weak self as obj => move |_, _| {
                obj.destroy();
            })
        );
        action!(
            self,
            "fullscreen",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_fullscreen();
            })
        );
    }

    fn setup_unit_system_text(&self, unit_system: UnitSystem) {
        let imp = self.imp();
        if unit_system == UnitSystem::Metric {
            imp.height_spin_button.set_value(
                Length::new::<inch>(imp.height_spin_button.value() as f32)
                    .get::<centimeter>()
                    .into(),
            );
        } else {
            imp.height_spin_button.set_value(
                Length::new::<centimeter>(imp.height_spin_button.value() as f32)
                    .get::<inch>()
                    .into(),
            );
        }
    }

    fn set_optimal_weight_goal(&self) {
        let imp = self.imp();

        let unitless_height = imp.height_spin_button.raw_value().unwrap_or_default();
        let height = if imp.current_unit_system.get() == UnitSystem::Metric {
            Length::new::<centimeter>(unitless_height)
        } else {
            Length::new::<inch>(unitless_height)
        };
        let optimal_value = Mass::new::<kilogram>(
            (OPTIMAL_BMI * height.get::<meter>() * height.get::<meter>()).round_decimal_places(1),
        );
        if imp.current_unit_system.get() == UnitSystem::Metric {
            imp.weight_goal_spin_button
                .set_value(optimal_value.get::<kilogram>().into());
        } else {
            imp.weight_goal_spin_button
                .set_value(optimal_value.get::<pound>().into());
        }
    }

    #[template_callback]
    fn try_enable_next_button_first_page(&self) {
        let imp = self.imp();
        let birthday = imp.birthday_selector.selected_date().reset_hms();
        let sensitive = birthday != glib::DateTime::local().reset_hms()
            && !imp.height_spin_button.has_default_value();
        imp.setup_next_page_button.set_sensitive(sensitive);
        imp.setup_carousel.set_interactive(sensitive);
    }

    #[template_callback]
    fn try_enable_next_button_third_page(&self) {
        let imp = self.imp();
        let sensitive = !imp.weight_spin_button.has_default_value();
        imp.setup_next_page_button.set_sensitive(sensitive);
        imp.setup_carousel.set_interactive(sensitive);
    }
}

#[cfg(test)]
mod test {
    use super::SetupWindow;
    use crate::{core::Application, utils::init_gtk};
    use gtk::{gio, prelude::*};

    #[test]
    fn new() {
        init_gtk();
        let app = Application::new();
        app.set_application_id(Some("dev.Cogitri.Health.Tests.SetupWindow.New"));
        app.register(None::<&gio::Cancellable>).unwrap();
        SetupWindow::new(&app);
    }
}
