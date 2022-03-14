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

use crate::model::Activity;
use adw::prelude::*;
use gtk::glib;

mod imp {
    use crate::{
        core::{i18n, ni18n_f, Settings, UnitSystem},
        model::{Activity, ActivityDataPoints, ActivityInfo},
        prelude::*,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{glib, subclass::prelude::*};
    use once_cell::unsync::OnceCell;
    use std::convert::TryInto;
    use uom::si::length::{meter, yard};

    #[derive(Debug, Default)]
    pub struct ActivityRow {
        pub activity: OnceCell<Activity>,
        pub settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityRow {
        const NAME: &'static str = "HealthActivityRow";
        type ParentType = adw::ExpanderRow;
        type Type = super::ActivityRow;
    }

    impl ObjectImpl for ActivityRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "activity",
                    "activity",
                    "activity",
                    Activity::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "activity" => {
                    let activity = value.get::<Activity>().unwrap();
                    let activity_info = ActivityInfo::from(activity.activity_type());

                    let minutes = activity.duration().as_minutes();
                    // TRANSLATORS: activity for x minutes, e.g. "Walking for 10 minutes",
                    obj.set_title(&ni18n_f(
                        "{} for {} Minute",
                        "{} for {} Minutes",
                        minutes.try_into().unwrap_or(0),
                        &[&activity_info.name, &minutes.to_string()],
                    ));
                    obj.set_subtitle(&activity.date().format_local());

                    if activity_info
                        .available_data_points
                        .contains(ActivityDataPoints::CALORIES_BURNED)
                    {
                        if let Some(calories_burned) = activity.calories_burned() {
                            obj.add_new_row(&i18n("Calories burned"), &calories_burned.to_string());
                        }
                    }

                    if activity_info
                        .available_data_points
                        .contains(ActivityDataPoints::HEART_RATE)
                    {
                        if activity.heart_rate_avg().unwrap_or(0) != 0 {
                            obj.add_new_row(
                                &i18n("Average heart rate"),
                                &activity.heart_rate_avg().unwrap().to_string(),
                            );
                        }
                        if activity.heart_rate_max().unwrap_or(0) != 0 {
                            obj.add_new_row(
                                &i18n("Maximum heart rate"),
                                &activity.heart_rate_max().unwrap().to_string(),
                            );
                        }
                        if activity.heart_rate_min().unwrap_or(0) != 0 {
                            obj.add_new_row(
                                &i18n("Minimum heart rate"),
                                &activity.heart_rate_min().unwrap().to_string(),
                            );
                        }
                    }

                    if activity_info
                        .available_data_points
                        .contains(ActivityDataPoints::DISTANCE)
                    {
                        if let Some(distance) = activity.distance() {
                            let args = if self.settings.unit_system() == UnitSystem::Metric {
                                let m = distance.get::<meter>().round_decimal_places(1);
                                ni18n_f("{} meter", "{} meters", m as u32, &[&m.to_string()])
                            } else {
                                let yards = distance.get::<yard>().round_decimal_places(1);
                                ni18n_f("{} yard", "{} yards", yards as u32, &[&yards.to_string()])
                            };
                            obj.add_new_row(&i18n("Distance"), &args);
                        }
                    }

                    self.activity.set(activity).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "activity" => self.activity.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ActivityRow {}
    impl ListBoxRowImpl for ActivityRow {}
    impl PreferencesRowImpl for ActivityRow {}
    impl ExpanderRowImpl for ActivityRow {}
}

glib::wrapper! {
    /// An implementation of [gtk::ListBox] that displays infos about an [Activity].
    pub struct ActivityRow(ObjectSubclass<imp::ActivityRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ExpanderRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ActivityRow {
    /// Create a new [ActivityRow].
    pub fn new(activity: &Activity) -> Self {
        glib::Object::new(&[("activity", activity)]).expect("Failed to create ActivityRow")
    }

    pub fn activity(&self) -> Activity {
        self.property("activity")
    }

    /// Set which [Activity] to display.
    pub fn set_activity(&self, activity: Activity) {
        self.set_property("activity", activity)
    }

    fn add_new_row(&self, title: &str, data: &str) {
        let row = adw::ActionRow::builder().title(title).build();
        row.add_suffix(&gtk::Label::builder().label(data).build());
        self.add_row(&row);
    }
}

#[cfg(test)]
mod test {
    use super::ActivityRow;
    use crate::{model::Activity, utils::init_gtk};
    use uom::si::{f32::Length, length::kilometer};

    #[test]
    fn new() {
        init_gtk();

        let act = Activity::new();
        act.set_calories_burned(Some(100));
        act.set_heart_rate_avg(Some(75));
        act.set_distance(Some(Length::new::<kilometer>(1.0)));
        ActivityRow::new(&act);
    }
}
