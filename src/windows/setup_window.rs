use crate::core::Database;
use gdk::subclass::prelude::ObjectSubclass;
use gio::prelude::*;

mod imp {
    use crate::{
        core::{i18n, settings::Unitsystem, utils::get_spinbutton_value, Settings},
        widgets::{BMILevelBar, SyncListBox},
    };
    use adw::PreferencesRowExt;
    use glib::{
        clone,
        subclass::{self, Signal},
    };
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use uom::si::{
        f32::{Length, Mass},
        length::{centimeter, inch, meter},
        mass::{kilogram, pound},
    };

    static OPTIMAL_BMI: f32 = 22.5;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/setup_window.ui")]
    pub struct SetupWindow {
        pub settings: Settings,

        #[template_child]
        pub bmi_levelbar: TemplateChild<BMILevelBar>,
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
        pub age_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub height_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub stepgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub weightgoal_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub unit_metric_togglebutton: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub height_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weightgoal_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub setup_carousel: TemplateChild<adw::Carousel>,
        #[template_child]
        pub sync_list_box: TemplateChild<SyncListBox>,
    }

    impl ObjectSubclass for SetupWindow {
        const NAME: &'static str = "HealthSetupWindow";
        type ParentType = adw::ApplicationWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::SetupWindow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: Settings::new(),
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
                age_spin_button: TemplateChild::default(),
                height_spin_button: TemplateChild::default(),
                stepgoal_spin_button: TemplateChild::default(),
                weightgoal_spin_button: TemplateChild::default(),
                unit_metric_togglebutton: TemplateChild::default(),
                height_actionrow: TemplateChild::default(),
                weightgoal_actionrow: TemplateChild::default(),
                setup_carousel: TemplateChild::default(),
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

    impl ObjectImpl for SetupWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.stepgoal_spin_button.set_value(10000.0);
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/dev/Cogitri/Health/custom.css");
            gtk::StyleContext::add_provider_for_display(
                &obj.get_display(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            self.connect_handlers(obj);
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("setup-done", &[], glib::Type::Unit).build()]);

            SIGNALS.as_ref()
        }
    }

    impl SetupWindow {
        fn try_enable_next_button(&self) {
            let age = self.age_spin_button.get_text().unwrap().to_string();
            let height = self.height_spin_button.get_text().unwrap().to_string();
            let sensitive = !age.is_empty() && age != "0" && !height.is_empty() && height != "0";
            self.setup_next_page_button.set_sensitive(sensitive);
            self.setup_carousel.set_interactive(sensitive);
        }

        fn set_optimal_weightgoal(&self) {
            let unitless_height = get_spinbutton_value(&self.height_spin_button);
            let height = if self.unit_metric_togglebutton.get_active() {
                Length::new::<centimeter>(unitless_height)
            } else {
                Length::new::<inch>(unitless_height)
            };
            let optimal_value =
                Mass::new::<kilogram>(OPTIMAL_BMI * height.get::<meter>() * height.get::<meter>());
            if self.unit_metric_togglebutton.get_active() {
                self.weightgoal_spin_button
                    .set_value(optimal_value.get::<kilogram>().into());
            } else {
                self.weightgoal_spin_button
                    .set_value(optimal_value.get::<pound>().into());
            }
        }

        fn connect_handlers(&self, obj: &super::SetupWindow) {
            self.unit_metric_togglebutton
                .connect_toggled(clone!(@weak obj => move |btn| {
                    let self_ = SetupWindow::from_instance(&obj);
                    if (btn.get_active()) {
                        self_.height_actionrow.set_title(Some(&i18n("Height in centimeters")));
                        self_.weightgoal_actionrow.set_title(Some(&i18n("Weightgoal in KG")));
                        self_.bmi_levelbar.set_unitsystem(Unitsystem::Metric);
                        self_.height_spin_button.set_value (Length::new::<inch>(self_.height_spin_button.get_value() as f32).get::<centimeter>().into());
                    } else {
                        self_.height_actionrow.set_title(Some(&i18n("Height in inch")));
                        self_.weightgoal_actionrow.set_title(Some(&i18n("Weightgoal in pounds")));
                        self_.bmi_levelbar.set_unitsystem(Unitsystem::Imperial);
                        self_.height_spin_button.set_value (Length::new::<centimeter>(self_.height_spin_button.get_value() as f32).get::<inch>().into());
                    }
                }));

            self.height_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = SetupWindow::from_instance(&obj);
                    self_.set_optimal_weightgoal();
                    self_.try_enable_next_button();

                    let unitless_height = get_spinbutton_value(&self_.height_spin_button);
                    let height = if self_.unit_metric_togglebutton.get_active() {
                        Length::new::<centimeter>(unitless_height)
                    } else {
                        Length::new::<inch>(unitless_height)
                    };
                    self_.bmi_levelbar.set_height(height);
                }));

            self.weightgoal_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = SetupWindow::from_instance(&obj);
                    let unitless_weight = get_spinbutton_value(&self_.weightgoal_spin_button);
                    let weight = if self_.unit_metric_togglebutton.get_active() {
                        Mass::new::<kilogram>(unitless_weight)
                    } else {
                        Mass::new::<pound>(unitless_weight)
                    };

                    self_.bmi_levelbar.set_weight(weight);
                }));

            self.age_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    SetupWindow::from_instance(&obj).try_enable_next_button();
                }));

            self.setup_done_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let self_ = SetupWindow::from_instance(&obj);
                    let unitless_height = get_spinbutton_value(&self_.height_spin_button);
                    let height = if self_.unit_metric_togglebutton.get_active() {
                        self_.settings.set_unitsystem(Unitsystem::Metric);
                        Length::new::<centimeter>(unitless_height)
                    } else {
                        self_.settings.set_unitsystem(Unitsystem::Imperial);
                        Length::new::<inch>(unitless_height)
                    };

                    self_.settings.set_user_age(get_spinbutton_value(&self_.age_spin_button));
                    self_.settings.set_user_height (height);
                    self_.settings.set_user_stepgoal(get_spinbutton_value(&self_.stepgoal_spin_button));

                    let unitless_weight = get_spinbutton_value(&self_.weightgoal_spin_button);
                    let weight = if self_.unit_metric_togglebutton.get_active() {
                        Mass::new::<kilogram>(unitless_weight)
                    } else {
                        Mass::new::<pound>(unitless_weight)
                    };
                    self_.settings.set_user_weightgoal(weight);

                    obj.emit("setup-done", &[]).unwrap();
                    obj.destroy();
                }));

            self.setup_quit_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    obj.destroy();
                }));

            self.setup_next_page_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let self_ = SetupWindow::from_instance(&obj);
                    match (self_.setup_carousel.get_position() as u32) {
                        0 => self_.setup_carousel.scroll_to (&self_.setup_second_page.get()),
                        1 => self_.setup_carousel.scroll_to (&self_.setup_third_page.get()),
                        2 => self_.setup_carousel.scroll_to (&self_.setup_fourth_page.get()),
                        3 => self_.setup_done_button.emit_clicked(),
                        _ => unimplemented!(),
                    }
                }));

            self.setup_previous_page_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let self_ = SetupWindow::from_instance(&obj);
                    match (self_.setup_carousel.get_position() as u32) {
                        0 => obj.destroy(),
                        1 => self_.setup_carousel.scroll_to (&self_.setup_first_page.get()),
                        2 => self_.setup_carousel.scroll_to (&self_.setup_second_page.get()),
                        3 => self_.setup_carousel.scroll_to (&self_.setup_third_page.get()),
                        _ => unimplemented!(),
                    }
                }));

            self.setup_carousel
                .connect_page_changed(clone!(@weak obj => move|carousel, index| {
                    let self_ = SetupWindow::from_instance(&obj);

                    if carousel.get_n_pages() -1 == index {
                        self_.setup_done_button.set_visible(true);
                        self_.setup_right_stack.set_visible_child(&self_.setup_done_button.get());
                    } else if (index == 0) {
                        self_.setup_quit_button.set_visible (true);
                        self_.setup_left_stack.set_visible_child(&self_.setup_quit_button.get());
                    } else {
                        self_.setup_next_page_button.set_visible(true);
                        self_.setup_previous_page_button.set_visible(true);
                        self_.setup_right_stack.set_visible_child(&self_.setup_next_page_button.get());
                        self_.setup_left_stack.set_visible_child(&self_.setup_previous_page_button.get());
                    }
                }));
        }
    }

    impl WidgetImpl for SetupWindow {}
    impl WindowImpl for SetupWindow {}
    impl ApplicationWindowImpl for SetupWindow {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for SetupWindow {}
}

glib::wrapper! {
    pub struct SetupWindow(ObjectSubclass<imp::SetupWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl SetupWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, db: Database) -> Self {
        let o = glib::Object::new(&[("application", app)]).expect("Failed to create SetupWindow");

        imp::SetupWindow::from_instance(&o)
            .sync_list_box
            .set_database(db);

        o
    }

    pub fn connect_setup_done<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("setup-done", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }
}