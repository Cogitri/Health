/* activity_row.rs
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
    core::{date::prelude::*, i18n_f, Unitsystem},
    model::{Activity, ActivityDataPoints, ActivityInfo},
};
use gtk::{
    glib::{self, subclass::prelude::*},
    prelude::*,
};
use uom::{
    fmt::DisplayStyle::Abbreviation,
    si::length::{meter, yard},
};

mod imp {
    use crate::{core::Settings, model::Activity};
    use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate, Default)]
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

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityRow {
        const NAME: &'static str = "HealthActivityRow";
        type ParentType = gtk::ListBoxRow;
        type Type = super::ActivityRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let gesture_controller = gtk::GestureClick::new();
            gesture_controller.connect_pressed(glib::clone!(@weak obj => move |_,_,_,_| {
                let self_ = obj.imp();
                self_.details_revealer.set_reveal_child(!self_.details_revealer.reveals_child());
            }));
        }
    }

    impl WidgetImpl for ActivityRow {}
    impl ListBoxRowImpl for ActivityRow {}
}

glib::wrapper! {
    /// An implementation of [gtk::ListBox] that displays infos about an [Activity].
    pub struct ActivityRow(ObjectSubclass<imp::ActivityRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl ActivityRow {
    /// Create a new [ActivityRow].
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ActivityRow")
    }

    /// Set which [Activity] to display.
    pub fn set_activity(&self, activity: Activity) {
        let self_ = self.imp();

        let activity_info = ActivityInfo::from(activity.activity_type());

        self_.active_minutes_label.set_label(&i18n_f(
            "{} Minutes",
            &[&activity.duration().num_minutes().to_string()],
        ));
        self_
            .activity_date_label
            .set_text(&activity.date().format_local());
        self_.activity_type_label.set_label(&activity_info.name);

        if activity_info
            .available_data_points
            .contains(ActivityDataPoints::CALORIES_BURNED)
        {
            if let Some(calories_burned) = activity.calories_burned() {
                self_
                    .calories_burned_label
                    .set_label(&i18n_f("{} Calories", &[&calories_burned.to_string()]));
            }
        }

        if activity_info
            .available_data_points
            .contains(ActivityDataPoints::HEART_RATE)
        {
            if activity.heart_rate_avg().unwrap_or(0) != 0 {
                self_
                    .heart_rate_average_label
                    .set_text(&activity.heart_rate_avg().unwrap().to_string());
                self_.heart_rate_average_row.set_visible(true);
            }
            if activity.heart_rate_max().unwrap_or(0) != 0 {
                self_
                    .heart_rate_maximum_label
                    .set_text(&activity.heart_rate_max().unwrap().to_string());
                self_.heart_rate_maximum_row.set_visible(true);
            }
            if activity.heart_rate_min().unwrap_or(0) != 0 {
                self_
                    .heart_rate_minimum_label
                    .set_text(&activity.heart_rate_min().unwrap().to_string());
                self_.heart_rate_minimum_row.set_visible(true);
            }
        }

        if activity_info
            .available_data_points
            .contains(ActivityDataPoints::DISTANCE)
        {
            if let Some(distance) = activity.distance() {
                self_.distance_row.set_visible(true);

                if self_.settings.unitsystem() == Unitsystem::Imperial {
                    self_.distance_label.set_label(&format!(
                        "{}",
                        distance.into_format_args(meter, Abbreviation)
                    ));
                } else {
                    self_.distance_label.set_label(&format!(
                        "{}",
                        distance.into_format_args(yard, Abbreviation)
                    ));
                };
            }
        }

        self_.activity.set(activity).unwrap();
    }

    fn imp(&self) -> &imp::ActivityRow {
        imp::ActivityRow::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use super::ActivityRow;
    use crate::{core::i18n_f, model::Activity};
    use gtk::prelude::WidgetExt;
    use uom::si::{f32::Length, length::kilometer};

    #[test]
    fn test_set_activity() {
        crate::utils::init_gtk();

        let act = Activity::new();
        act.set_calories_burned(Some(100));
        act.set_heart_rate_avg(Some(75));
        act.set_distance(Some(Length::new::<kilometer>(1.0)));
        let row = ActivityRow::new();
        let row_priv = row.imp();
        row.set_activity(act);

        assert_eq!(
            row_priv.calories_burned_label.label().as_str(),
            i18n_f("{} Calories", &[&100.to_string()]).as_str()
        );
        assert_eq!(
            row_priv.heart_rate_average_label.label().as_str(),
            75.to_string().as_str(),
        );
        assert!(row_priv
            .heart_rate_minimum_label
            .label()
            .as_str()
            .is_empty());
        assert!(row_priv
            .heart_rate_maximum_label
            .label()
            .as_str()
            .is_empty());
        assert!(row_priv.distance_row.get_visible());
    }
}
