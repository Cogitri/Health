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

use crate::core::{utils::prelude::*, UnitSystem};
use adw::prelude::*;
use chrono::Local;
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
        core::{Settings, UnitSystem},
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

        #[template_child]
        pub bmi_levelbar: TemplateChild<BmiLevelBar>,
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
        pub birthday_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub height_spin_button: TemplateChild<UnitSpinButton>,
        #[template_child]
        pub step_goal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub weight_goal_spin_button: TemplateChild<UnitSpinButton>,
        #[template_child]
        pub unit_imperial_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub unit_metric_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_goal_actionrow: TemplateChild<adw::ActionRow>,
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
            Self {
                current_unit_system: Cell::new(settings.unit_system()),
                settings,
                bmi_levelbar: TemplateChild::default(),
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
                birthday_selector: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                step_goal_spin_button: TemplateChild::default(),
                weight_goal_spin_button: TemplateChild::default(),
                unit_imperial_togglebutton: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                height_actionrow: TemplateChild::default(),
                weight_goal_actionrow: TemplateChild::default(),
                setup_carousel: TemplateChild::default(),
                sync_list_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            UnitSpinButton::static_type();
            Self::bind_template(klass);
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

impl SetupWindow {
    /// Connect to the setup being completed by the user.
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_setup_done<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("setup-done", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    /// Create a new [SetupWindow].
    ///
    /// # Arguments
    /// * `app` - The [GtkApplication](gtk::Application) to use.
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::new(&[("application", app)]).expect("Failed to create SetupWindow")
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_.birthday_selector.connect_selected_date_notify(
            clone!(@weak self as obj => move |_| {
                obj.try_enable_next_button();
            }),
        );

        self_
            .height_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_height_spin_button_changed();
            }));

        self_.setup_carousel.connect_page_changed(
            clone!(@weak self as obj => move|carousel, index| {
                obj.handle_setup_carousel_page_changed(carousel, index);
            }),
        );

        self_
            .setup_done_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_setup_done_button_clicked();
            }));

        self_
            .setup_next_page_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_setup_next_page_button_clicked();
            }));

        self_
            .setup_previous_page_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_setup_previous_page_button_clicked();
            }));

        self_
            .setup_quit_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.destroy();
            }));

        self_
            .settings
            .connect_unit_system_changed(clone!(@weak self as obj => move |_, _| {
                obj.handle_unit_system_changed();
            }));

        self_
            .weight_goal_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_weight_spin_button_changed();
            }));
    }

    fn imp(&self) -> &imp::SetupWindow {
        imp::SetupWindow::from_instance(self)
    }

    fn handle_fullscreen(&self) {
        if self.is_fullscreen() {
            self.unfullscreen();
        } else {
            self.fullscreen();
        }
    }

    fn handle_height_spin_button_changed(&self) {
        let self_ = self.imp();
        self.set_optimal_weight_goal();
        self.try_enable_next_button();

        let unitless_height = self_.height_spin_button.raw_value().unwrap_or_default();
        let height = if self_.current_unit_system.get() == UnitSystem::Metric {
            Length::new::<centimeter>(unitless_height)
        } else {
            Length::new::<inch>(unitless_height)
        };
        self_.bmi_levelbar.set_height(height);
    }

    fn handle_setup_carousel_page_changed(&self, carousel: &adw::Carousel, index: u32) {
        let self_ = self.imp();

        if carousel.n_pages() - 1 == index {
            self_.setup_done_button.set_visible(true);
            self_
                .setup_right_stack
                .set_visible_child(&self_.setup_done_button.get());
        } else if index == 0 {
            self_.setup_quit_button.set_visible(true);
            self_
                .setup_left_stack
                .set_visible_child(&self_.setup_quit_button.get());
        } else {
            self_.setup_next_page_button.set_visible(true);
            self_.setup_previous_page_button.set_visible(true);
            self_
                .setup_right_stack
                .set_visible_child(&self_.setup_next_page_button.get());
            self_
                .setup_left_stack
                .set_visible_child(&self_.setup_previous_page_button.get());
        }
    }

    fn handle_setup_done_button_clicked(&self) {
        let self_ = self.imp();
        let unitless_height = self_.height_spin_button.raw_value().unwrap_or_default();
        let height = if self_.current_unit_system.get() == UnitSystem::Metric {
            self_.settings.set_unit_system(UnitSystem::Metric);
            Length::new::<centimeter>(unitless_height)
        } else {
            self_.settings.set_unit_system(UnitSystem::Imperial);
            Length::new::<inch>(unitless_height)
        };

        self_
            .settings
            .set_user_birthday(self_.birthday_selector.selected_date().date());
        self_.settings.set_user_height(height);
        self_
            .settings
            .set_user_step_goal(self_.step_goal_spin_button.raw_value().unwrap_or_default());

        let unitless_weight = self_
            .weight_goal_spin_button
            .raw_value()
            .unwrap_or_default();
        let weight = if self_.current_unit_system.get() == UnitSystem::Metric {
            Mass::new::<kilogram>(unitless_weight)
        } else {
            Mass::new::<pound>(unitless_weight)
        };
        self_.settings.set_user_weight_goal(weight);

        self.emit_by_name("setup-done", &[]).unwrap();
        self.destroy();
    }

    fn handle_setup_next_page_button_clicked(&self) {
        let self_ = self.imp();
        match self_.setup_carousel.position() as u32 {
            0 => self_
                .setup_carousel
                .scroll_to(&self_.setup_second_page.get()),
            1 => self_
                .setup_carousel
                .scroll_to(&self_.setup_third_page.get()),
            2 => self_
                .setup_carousel
                .scroll_to(&self_.setup_fourth_page.get()),
            3 => self_.setup_done_button.emit_clicked(),
            _ => unimplemented!(),
        }
    }

    fn handle_setup_previous_page_button_clicked(&self) {
        let self_ = self.imp();
        match self_.setup_carousel.position() as u32 {
            0 => self.destroy(),
            1 => self_
                .setup_carousel
                .scroll_to(&self_.setup_first_page.get()),
            2 => self_
                .setup_carousel
                .scroll_to(&self_.setup_second_page.get()),
            3 => self_
                .setup_carousel
                .scroll_to(&self_.setup_third_page.get()),
            _ => unimplemented!(),
        }
    }

    fn handle_weight_spin_button_changed(&self) {
        let self_ = self.imp();
        let unitless_weight = self_
            .weight_goal_spin_button
            .raw_value()
            .unwrap_or_default();
        let weight = if self_.current_unit_system.get() == UnitSystem::Metric {
            Mass::new::<kilogram>(unitless_weight)
        } else {
            Mass::new::<pound>(unitless_weight)
        };

        self_.bmi_levelbar.set_weight(weight);
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
        let self_ = self.imp();
        if unit_system == UnitSystem::Metric {
            self_.height_spin_button.set_value(
                Length::new::<inch>(self_.height_spin_button.value() as f32)
                    .get::<centimeter>()
                    .into(),
            );
        } else {
            self_.height_spin_button.set_value(
                Length::new::<centimeter>(self_.height_spin_button.value() as f32)
                    .get::<inch>()
                    .into(),
            );
        }
    }

    fn set_optimal_weight_goal(&self) {
        let self_ = self.imp();

        let unitless_height = self_.height_spin_button.raw_value().unwrap_or_default();
        let height = if self_.current_unit_system.get() == UnitSystem::Metric {
            Length::new::<centimeter>(unitless_height)
        } else {
            Length::new::<inch>(unitless_height)
        };
        let optimal_value = Mass::new::<kilogram>(
            (OPTIMAL_BMI * height.get::<meter>() * height.get::<meter>()).round_decimal_places(1),
        );
        if self_.current_unit_system.get() == UnitSystem::Metric {
            self_
                .weight_goal_spin_button
                .set_value(optimal_value.get::<kilogram>().into());
        } else {
            self_
                .weight_goal_spin_button
                .set_value(optimal_value.get::<pound>().into());
        }
    }

    fn try_enable_next_button(&self) {
        let self_ = self.imp();
        let birthday = self_.birthday_selector.selected_date().date();
        let height = self_.height_spin_button.text().to_string();
        let sensitive = birthday != Local::now().date() && !height.is_empty() && height != "0";
        self_.setup_next_page_button.set_sensitive(sensitive);
        self_.setup_carousel.set_interactive(sensitive);
    }
}
