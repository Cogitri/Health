/* view_add_activity.rs
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
    core::utils::prelude::*,
    model::{Activity, ActivityDataPoints, ActivityInfo, Unitsize},
    views::View,
};
use chrono::Duration;
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*},
    prelude::*,
};
use gtk_macros::stateful_action;
use imp::spin_button_value_if_datapoint;
use std::str::FromStr;

mod imp {
    use crate::{
        core::{utils::prelude::*, Database, Settings},
        model::{Activity, ActivityDataPoints, ActivityInfo, ActivityType},
        views::{PinnedResultFuture, View, ViewImpl},
        widgets::{ActivityTypeSelector, DateSelector, DistanceActionRow},
    };
    use gtk::{
        gio,
        glib::{self, clone},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ViewAddActivityMut {
        pub activity: Activity,
        pub user_changed_datapoints: ActivityDataPoints,
        pub filter_model: Option<gtk::FilterListModel>,
        pub selected_activity: ActivityInfo,
        pub stop_update: bool,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_add_dialog.ui")]
    pub struct ViewAddActivity {
        pub inner: RefCell<ViewAddActivityMut>,
        pub database: Database,
        pub settings: Settings,

        #[template_child]
        pub activity_type_selector: TemplateChild<ActivityTypeSelector>,
        #[template_child]
        pub date_selector: TemplateChild<DateSelector>,
        #[template_child]
        pub activities_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub activity_type_menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub calories_burned_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub heart_rate_average_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub heart_rate_max_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub heart_rate_min_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub steps_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub activity_type_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub calories_burned_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub date_selector_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub distance_action_row: TemplateChild<DistanceActionRow>,
        #[template_child]
        pub duration_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_average_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_max_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub heart_rate_min_action_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub stepcount_action_row: TemplateChild<adw::ActionRow>,
    }

    /// Get the value of a spinbutton if the datapoint is set.
    ///
    /// # Parameters
    /// * `spin_button` - The [GtkSpinButton](gtk::SpinButton) whose value to retrieve.
    /// * `activity` - The [ActivityInfo] that describes the current activity.
    /// * `datapoints` - The [ActivityDataPoints] to check for.
    ///
    /// # Returns
    /// `Some` with the value of the [GtkSpinButton](gtk::SpinButton) if the activity has that
    /// particular datapoint, or `None`.
    pub fn spin_button_value_if_datapoint(
        spin_button: &gtk::SpinButton,
        activity: &ActivityInfo,
        datapoints: ActivityDataPoints,
    ) -> Option<u32> {
        if activity.available_data_points.contains(datapoints) && spin_button.text().as_str() != ""
        {
            Some(spin_button.raw_value().unwrap_or_default())
        } else {
            None
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewAddActivity {
        const NAME: &'static str = "HealthViewAddActivity";
        type ParentType = View;
        type Type = super::ViewAddActivity;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ViewAddActivity {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let model = gio::ListStore::new(gtk::Widget::static_type());
            model.splice(
                0,
                0,
                &[
                    self.date_selector_actionrow.get(),
                    self.activity_type_actionrow.get(),
                    self.calories_burned_action_row.get(),
                    self.distance_action_row.get().upcast(),
                    self.duration_action_row.get(),
                    self.heart_rate_average_action_row.get(),
                    self.heart_rate_min_action_row.get(),
                    self.heart_rate_max_action_row.get(),
                    self.stepcount_action_row.get(),
                ],
            );

            let filter = gtk::CustomFilter::new(clone!(@weak obj => @default-panic, move |o| {
                obj.filter_activity_entry(o)
            }));
            let filter_model = gtk::FilterListModel::new(Some(&model), Some(&filter));
            self.activities_list_box
                .bind_model(Some(&filter_model), |o| {
                    o.clone().downcast::<gtk::Widget>().unwrap()
                });

            self.inner.borrow_mut().filter_model = Some(filter_model);
            obj.connect_handlers();
            obj.set_selected_activity(ActivityInfo::from(ActivityType::Walking));
            obj.setup_actions();
        }
    }

    impl WidgetImpl for ViewAddActivity {}

    impl ViewImpl for ViewAddActivity {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }
    }
}

glib::wrapper! {
    /// A few widgets for adding a new activity record.
    pub struct ViewAddActivity(ObjectSubclass<imp::ViewAddActivity>)
        @extends gtk::Widget, View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewAddActivity {
    /// Create a new [ViewAddActivity].
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewAddActivity")
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_
            .calories_burned_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_calories_burned_spin_button_changed();
            }));
        self_
            .distance_action_row
            .connect_changed(clone!(@weak self as obj => move || {
                obj.handle_distance_action_row_changed();
            }));
        self_
            .duration_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_duration_spin_button_changed();
            }));
        self_
            .steps_spin_button
            .connect_changed(clone!(@weak self as obj => move |_| {
                obj.handle_steps_spin_button_changed();
            }));

        self_
            .activity_type_selector
            .connect_selected_activity_notify(clone!(@weak self as obj => move |_| {
                obj.handle_activity_type_selector_activity_selected();
            }));

        self_.calories_burned_spin_button.connect_input(
            clone!(@weak self as obj => @default-panic, move |_| {
                obj.handle_calories_burned_spin_button_input()
            }),
        );
        self_
            .distance_action_row
            .connect_input(clone!(@weak self as obj => move || {
                obj.handle_distance_action_row_input()
            }));
        self_.duration_spin_button.connect_input(
            clone!(@weak self as obj => @default-panic, move |_| {
                obj.handle_duration_spin_button_input()
            }),
        );
        self_.steps_spin_button.connect_input(
            clone!(@weak self as obj => @default-panic, move |_| {
                obj.handle_steps_spin_button_input()
            }),
        );
    }

    fn imp(&self) -> &imp::ViewAddActivity {
        imp::ViewAddActivity::from_instance(self)
    }

    fn setup_actions(&self) {
        let action_group = gio::SimpleActionGroup::new();

        stateful_action!(
            action_group,
            "unitsize",
            Some(&String::static_variant_type()),
            "small",
            clone!(@weak self as obj => move |a, p| {
                let parameter = p.unwrap();

                obj.imp().distance_action_row.set_unitsize(Unitsize::from_str(parameter.get::<String>().unwrap().as_str()).unwrap());

                a.set_state(parameter);
            })
        );

        self.insert_action_group("activity_add_dialog", Some(&action_group));
    }

    fn set_selected_activity(&self, val: ActivityInfo) {
        self.imp().activity_type_menu_button.set_label(&val.name);
    }

    fn filter_activity_entry(&self, o: &glib::Object) -> bool {
        let self_ = self.imp();

        let datapoints = self_
            .activity_type_selector
            .selected_activity()
            .available_data_points;

        if let Some(row) = o.downcast_ref::<adw::ActionRow>() {
            if (row == &self_.activity_type_actionrow.get()
                || row == &self_.date_selector_actionrow.get())
                || (row == &self_.calories_burned_action_row.get()
                    && datapoints.contains(ActivityDataPoints::CALORIES_BURNED))
                || (row == &self_.distance_action_row.get()
                    && datapoints.contains(ActivityDataPoints::DISTANCE))
                || (row == &self_.duration_action_row.get()
                    && datapoints.contains(ActivityDataPoints::DURATION))
                || (row == &self_.stepcount_action_row.get()
                    && datapoints.contains(ActivityDataPoints::STEP_COUNT))
                || ((row == &self_.heart_rate_average_action_row.get()
                    || row == &self_.heart_rate_max_action_row.get()
                    || row == &self_.heart_rate_min_action_row.get())
                    && datapoints.contains(ActivityDataPoints::HEART_RATE))
            {
                return true;
            }
        }

        false
    }

    fn handle_activity_type_selector_activity_selected(&self) {
        let self_ = self.imp();
        self.set_selected_activity(self_.activity_type_selector.selected_activity());
        let inner = self_.inner.borrow_mut();
        inner
            .activity
            .set_activity_type(inner.selected_activity.activity_type.clone());

        if let Some(model) = &inner.filter_model {
            if let Some(filter) = model.filter() {
                filter.changed(gtk::FilterChange::Different);
            }
        }
    }

    fn handle_calories_burned_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.imp()
            .inner
            .borrow_mut()
            .user_changed_datapoints
            .insert(ActivityDataPoints::CALORIES_BURNED);
        None
    }

    fn handle_calories_burned_spin_button_changed(&self) {
        let self_ = self.imp();
        {
            let activity = &self_.inner.borrow_mut().activity;
            activity.set_calories_burned(Some(
                self_
                    .calories_burned_spin_button
                    .raw_value()
                    .unwrap_or_default(),
            ));
            activity.autofill_from_calories();
        }
        self.set_spin_buttons_from_activity(self_.calories_burned_spin_button.upcast_ref());
    }

    fn handle_distance_action_row_changed(&self) {
        let self_ = self.imp();
        {
            let activity = &self_.inner.borrow_mut().activity;
            activity.set_distance(Some(self_.distance_action_row.value()));
            activity.autofill_from_distance();
        }
        self.set_spin_buttons_from_activity(self_.distance_action_row.upcast_ref());
    }

    fn handle_distance_action_row_input(&self) {
        self.imp()
            .inner
            .borrow_mut()
            .user_changed_datapoints
            .insert(ActivityDataPoints::DISTANCE);
    }

    fn handle_duration_spin_button_changed(&self) {
        let self_ = self.imp();
        {
            let activity = &self_.inner.borrow_mut().activity;
            activity.set_duration(Duration::minutes(
                self_.duration_spin_button.raw_value().unwrap_or_default(),
            ));
            activity.autofill_from_minutes();
        }
        self.set_spin_buttons_from_activity(self_.duration_spin_button.upcast_ref());
    }

    fn handle_duration_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.imp()
            .inner
            .borrow_mut()
            .user_changed_datapoints
            .insert(ActivityDataPoints::DURATION);
        None
    }

    fn handle_steps_spin_button_changed(&self) {
        let self_ = self.imp();
        {
            let activity = &self_.inner.borrow_mut().activity;
            activity.set_steps(Some(
                self_.steps_spin_button.raw_value().unwrap_or_default(),
            ));
            activity.autofill_from_steps();
        }
        self.set_spin_buttons_from_activity(self_.steps_spin_button.upcast_ref());
    }

    fn handle_steps_spin_button_input(&self) -> Option<Result<f64, ()>> {
        self.imp()
            .inner
            .borrow_mut()
            .user_changed_datapoints
            .insert(ActivityDataPoints::STEP_COUNT);
        None
    }

    pub async fn handle_response(&self, id: gtk::ResponseType) {
        if id == gtk::ResponseType::Ok {
            let self_ = self.imp();
            let selected_activity = self_.activity_type_selector.selected_activity();
            let distance = if selected_activity
                .available_data_points
                .contains(ActivityDataPoints::DISTANCE)
            {
                Some(self_.distance_action_row.value())
            } else {
                None
            };

            let activity = Activity::new();
            activity
                .set_date(self_.date_selector.selected_date())
                .set_activity_type(selected_activity.activity_type.clone())
                .set_calories_burned(spin_button_value_if_datapoint(
                    &self_.calories_burned_spin_button,
                    &selected_activity,
                    ActivityDataPoints::CALORIES_BURNED,
                ))
                .set_distance(distance)
                .set_heart_rate_avg(spin_button_value_if_datapoint(
                    &self_.heart_rate_average_spin_button,
                    &selected_activity,
                    ActivityDataPoints::HEART_RATE,
                ))
                .set_heart_rate_min(spin_button_value_if_datapoint(
                    &self_.heart_rate_min_spin_button,
                    &selected_activity,
                    ActivityDataPoints::HEART_RATE,
                ))
                .set_heart_rate_max(spin_button_value_if_datapoint(
                    &self_.heart_rate_max_spin_button,
                    &selected_activity,
                    ActivityDataPoints::HEART_RATE,
                ))
                .set_steps(spin_button_value_if_datapoint(
                    &self_.steps_spin_button,
                    &selected_activity,
                    ActivityDataPoints::STEP_COUNT,
                ))
                .set_duration(Duration::minutes(
                    spin_button_value_if_datapoint(
                        &self_.calories_burned_spin_button,
                        &selected_activity,
                        ActivityDataPoints::DURATION,
                    )
                    .unwrap_or(0)
                    .into(),
                ));

            if let Err(e) = self_.database.save_activity(activity).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to save new data due to error {}",
                    e.to_string()
                )
            }
            self.save_recent_activity();
        }
    }

    fn save_recent_activity(&self) {
        let self_ = self.imp();
        let inner = self_.inner.borrow();

        let mut recent_activities = self_.settings.recent_activity_types();
        if !recent_activities
            .iter()
            .any(|s| inner.selected_activity.id == s)
        {
            recent_activities.push(inner.selected_activity.id.to_string());
            if recent_activities.len() > 4 {
                self_.settings.set_recent_activity_types(
                    &recent_activities[1..recent_activities.len()]
                        .iter()
                        .map(std::string::String::as_str)
                        .collect::<Vec<&str>>(),
                );
            } else {
                self_.settings.set_recent_activity_types(
                    &recent_activities
                        .iter()
                        .map(std::string::String::as_str)
                        .collect::<Vec<&str>>(),
                );
            }
        }
    }

    #[allow(clippy::unnecessary_unwrap)]
    fn set_spin_buttons_from_activity(&self, emitter: &gtk::Widget) {
        let self_ = self.imp();
        let (
            calories,
            calories_changed,
            distance,
            distance_changed,
            minutes,
            minutes_changed,
            steps,
            steps_changed,
        ) = {
            let mut inner = self_.inner.borrow_mut();
            if inner.stop_update {
                return;
            }

            inner.stop_update = true;

            (
                inner.activity.calories_burned().unwrap_or(0),
                inner
                    .user_changed_datapoints
                    .contains(ActivityDataPoints::CALORIES_BURNED),
                inner.activity.distance(),
                inner
                    .user_changed_datapoints
                    .contains(ActivityDataPoints::DISTANCE),
                inner.activity.duration().num_minutes(),
                inner
                    .user_changed_datapoints
                    .contains(ActivityDataPoints::DURATION),
                inner.activity.steps().unwrap_or(0),
                inner
                    .user_changed_datapoints
                    .contains(ActivityDataPoints::STEP_COUNT),
            )
        };

        if calories != 0
            && calories
                != self_
                    .calories_burned_spin_button
                    .raw_value::<u32>()
                    .unwrap_or_default()
            && self_
                .calories_burned_action_row
                .get()
                .upcast_ref::<gtk::Widget>()
                != emitter
            && !calories_changed
        {
            self_.calories_burned_spin_button.set_value(calories.into());
        }
        if distance.is_some()
            && distance != Some(self_.distance_action_row.value())
            && self_.distance_action_row.get().upcast_ref::<gtk::Widget>() != emitter
            && !distance_changed
        {
            self_.distance_action_row.set_value(distance.unwrap());
        }
        if minutes != 0
            && minutes
                != self_
                    .duration_spin_button
                    .raw_value::<i64>()
                    .unwrap_or_default()
            && self_.duration_action_row.get().upcast_ref::<gtk::Widget>() != emitter
            && !minutes_changed
        {
            self_.duration_spin_button.set_value(minutes as f64);
        }
        if steps != 0
            && steps
                != self_
                    .steps_spin_button
                    .raw_value::<u32>()
                    .unwrap_or_default()
            && self_.stepcount_action_row.get().upcast_ref::<gtk::Widget>() != emitter
            && !steps_changed
        {
            self_.steps_spin_button.set_value(steps.into());
        }

        self_.inner.borrow_mut().stop_update = false;
    }
}
