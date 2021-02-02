use crate::model::Activity;
use gdk::subclass::prelude::ObjectSubclass;

mod imp {
    use crate::{
        core::{i18n_f, settings::Unitsystem, Settings},
        model::{Activity, ActivityDataPoints, ActivityInfo},
    };
    use glib::subclass;
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use uom::fmt::DisplayStyle::Abbreviation;
    use uom::si::length::{meter, yard};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_row.ui")]
    pub struct ActivityRow {
        pub activity: OnceCell<Activity>,
        pub settings: Settings,
        #[template_child]
        pub active_minutes_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub activity_date_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub activity_type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub calories_burned_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub distance_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub heart_rate_average_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub heart_rate_maximum_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub heart_rate_minimum_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub steps_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub calories_burned_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub distance_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_average_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_maximum_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_minimum_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub steps_row: TemplateChild<adw::ActionRow>,
    }

    impl ObjectSubclass for ActivityRow {
        const NAME: &'static str = "HealthActivityRow";
        type ParentType = gtk::ListBoxRow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ActivityRow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                activity: OnceCell::new(),
                settings: Settings::new(),
                active_minutes_label: TemplateChild::default(),
                activity_date_label: TemplateChild::default(),
                activity_type_label: TemplateChild::default(),
                calories_burned_label: TemplateChild::default(),
                distance_label: TemplateChild::default(),
                heart_rate_average_label: TemplateChild::default(),
                heart_rate_maximum_label: TemplateChild::default(),
                heart_rate_minimum_label: TemplateChild::default(),
                steps_label: TemplateChild::default(),
                details_revealer: TemplateChild::default(),
                calories_burned_row: TemplateChild::default(),
                distance_row: TemplateChild::default(),
                heart_rate_average_row: TemplateChild::default(),
                heart_rate_maximum_row: TemplateChild::default(),
                heart_rate_minimum_row: TemplateChild::default(),
                steps_row: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let gesture_controller = gtk::GestureClick::new();
            gesture_controller.connect_pressed(glib::clone!(@weak obj => move |_,_,_,_| {
                let self_ = ActivityRow::from_instance(&obj);
                self_.details_revealer.set_reveal_child(!self_.details_revealer.get_reveal_child());
            }));
        }
    }

    impl WidgetImpl for ActivityRow {}
    impl ListBoxRowImpl for ActivityRow {}

    impl ActivityRow {
        pub fn set_activity(&self, activity: Activity) {
            let activity_info = ActivityInfo::from(activity.get_activity_type());

            self.active_minutes_label.set_label(&i18n_f(
                "{} Minutes",
                &[&activity.get_duration().num_minutes().to_string()],
            ));
            self.activity_date_label
                .set_text(&format!("{}", activity.get_date().format("%x")));
            self.activity_type_label.set_label(&activity_info.name);

            if activity_info
                .available_data_points
                .contains(ActivityDataPoints::CALORIES_BURNED)
            {
                if let Some(calories_burned) = activity.get_calories_burned() {
                    self.calories_burned_label
                        .set_label(&i18n_f("{} Calories", &[&calories_burned.to_string()]));
                }
            }

            if activity_info
                .available_data_points
                .contains(ActivityDataPoints::HEART_RATE)
            {
                if activity.get_heart_rate_avg().unwrap_or(0) != 0 {
                    self.heart_rate_average_label
                        .set_text(&activity.get_heart_rate_avg().unwrap().to_string());
                    self.heart_rate_average_row.set_visible(true);
                }
                if activity.get_heart_rate_max().unwrap_or(0) != 0 {
                    self.heart_rate_maximum_label
                        .set_text(&activity.get_heart_rate_max().unwrap().to_string());
                    self.heart_rate_maximum_row.set_visible(true);
                }
                if activity.get_heart_rate_min().unwrap_or(0) != 0 {
                    self.heart_rate_minimum_label
                        .set_text(&activity.get_heart_rate_min().unwrap().to_string());
                    self.heart_rate_minimum_row.set_visible(true);
                }
            }

            if activity_info
                .available_data_points
                .contains(ActivityDataPoints::DISTANCE)
            {
                if let Some(distance) = activity.get_distance() {
                    self.distance_row.set_visible(true);

                    if self.settings.get_unitsystem() == Unitsystem::Imperial {
                        self.distance_label.set_label(&format!(
                            "{}",
                            distance.clone().into_format_args(meter, Abbreviation)
                        ));
                    } else {
                        self.distance_label.set_label(&format!(
                            "{}",
                            distance.clone().into_format_args(yard, Abbreviation)
                        ));
                    };
                }
            }

            self.activity.set(activity).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct ActivityRow(ObjectSubclass<imp::ActivityRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl ActivityRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ActivityRow")
    }

    pub fn set_activity(&self, activity: Activity) {
        imp::ActivityRow::from_instance(&self).set_activity(activity);
    }
}
