use crate::core::HealthDatabase;
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use crate::{
        core::{utils::get_spinbutton_value, HealthSettings},
        model::{Activity, ActivityDataPoints, ActivityInfo, ActivityType},
        widgets::{HealthActivityTypeSelector, HealthDateSelector, HealthDistanceActionRow},
    };
    use chrono::Duration;
    use glib::{clone, subclass};
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct HealthActivityAddDialogMut {
        activity: Activity,
        calories_burned_spin_button_user_changed: bool,
        database: Option<HealthDatabase>,
        distance_spin_button_user_changed: bool,
        duration_spin_button_user_changed: bool,
        filter_model: Option<gtk::FilterListModel>,
        selected_activity: ActivityInfo,
        steps_spin_button_user_changed: bool,
        stop_update: bool,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_add_dialog.ui")]
    pub struct HealthActivityAddDialog {
        inner: RefCell<HealthActivityAddDialogMut>,
        pub settings: HealthSettings,

        #[template_child]
        pub activity_type_selector: TemplateChild<HealthActivityTypeSelector>,
        #[template_child]
        pub date_selector: TemplateChild<HealthDateSelector>,
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
        pub distance_action_row: TemplateChild<HealthDistanceActionRow>,
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

    fn get_spin_button_value_if_datapoint(
        b: &gtk::SpinButton,
        a: &ActivityInfo,
        d: ActivityDataPoints,
    ) -> Option<u32> {
        if a.available_data_points.contains(d) && b.get_text().unwrap().as_str() != "" {
            Some(get_spinbutton_value(b))
        } else {
            None
        }
    }

    impl ObjectSubclass for HealthActivityAddDialog {
        const NAME: &'static str = "HealthActivityAddDialog";
        type ParentType = gtk::Dialog;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthActivityAddDialog;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(HealthActivityAddDialogMut {
                    activity: Activity::new(),
                    calories_burned_spin_button_user_changed: false,
                    database: None,
                    distance_spin_button_user_changed: false,
                    duration_spin_button_user_changed: false,
                    filter_model: None,
                    selected_activity: ActivityInfo::from(ActivityType::Walking),
                    steps_spin_button_user_changed: false,
                    stop_update: false,
                }),
                settings: HealthSettings::new(),
                date_selector: TemplateChild::default(),
                activities_list_box: TemplateChild::default(),
                activity_type_actionrow: TemplateChild::default(),
                activity_type_menu_button: TemplateChild::default(),
                activity_type_selector: TemplateChild::default(),
                calories_burned_action_row: TemplateChild::default(),
                calories_burned_spin_button: TemplateChild::default(),
                date_selector_actionrow: TemplateChild::default(),
                distance_action_row: TemplateChild::default(),
                duration_action_row: TemplateChild::default(),
                duration_spin_button: TemplateChild::default(),
                heart_rate_average_action_row: TemplateChild::default(),
                heart_rate_average_spin_button: TemplateChild::default(),
                heart_rate_max_action_row: TemplateChild::default(),
                heart_rate_max_spin_button: TemplateChild::default(),
                heart_rate_min_action_row: TemplateChild::default(),
                heart_rate_min_spin_button: TemplateChild::default(),
                stepcount_action_row: TemplateChild::default(),
                steps_spin_button: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HealthActivityAddDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let model = gio::ListStore::new(gtk::Widget::static_type());
            model.splice(
                0,
                0,
                &[
                    self.date_selector_actionrow.get().upcast(),
                    self.activity_type_actionrow.get().upcast(),
                    self.calories_burned_action_row.get().upcast(),
                    self.distance_action_row.get().upcast(),
                    self.duration_action_row.get().upcast(),
                    self.heart_rate_average_action_row.get().upcast(),
                    self.heart_rate_min_action_row.get().upcast(),
                    self.heart_rate_max_action_row.get().upcast(),
                    self.stepcount_action_row.get().upcast(),
                ],
            );

            let filter = gtk::CustomFilter::new(clone!(@weak obj => move |o| {
                imp::HealthActivityAddDialog::from_instance(&obj).filter_activity_entry(o)
            }));
            let filter_model = gtk::FilterListModel::new(Some(&model), Some(&filter));
            self.activities_list_box
                .bind_model(Some(&filter_model), |o| {
                    o.clone().downcast::<gtk::Widget>().unwrap()
                });

            self.inner.borrow_mut().filter_model = Some(filter_model);
            self.connect_handlers(obj);
            self.set_selected_activity(ActivityInfo::from(ActivityType::Walking));
        }
    }

    impl WidgetImpl for HealthActivityAddDialog {}
    impl WindowImpl for HealthActivityAddDialog {}
    impl DialogImpl for HealthActivityAddDialog {}

    impl HealthActivityAddDialog {
        pub fn set_database(&self, database: HealthDatabase) {
            self.inner.borrow_mut().database = Some(database);
        }

        fn connect_handlers(&self, obj: &super::HealthActivityAddDialog) {
            obj.connect_response(|obj, id| match id {
                gtk::ResponseType::Ok => {
                    let downgraded = obj.downgrade();
                    glib::MainContext::default().spawn_local(async move {
                        if let Some(obj) = downgraded.upgrade() {
                            let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                            let selected_activity =
                                self_.activity_type_selector.get_selected_activity();
                            let distance = if selected_activity
                                .available_data_points
                                .contains(ActivityDataPoints::DISTANCE)
                            {
                                Some(self_.distance_action_row.get_value())
                            } else {
                                None
                            };

                            let activity = Activity::new();
                            activity
                                .set_activity_type(selected_activity.activity_type.clone())
                                .set_calories_burned(get_spin_button_value_if_datapoint(
                                    &self_.calories_burned_spin_button,
                                    &selected_activity,
                                    ActivityDataPoints::CALORIES_BURNED,
                                ))
                                .set_distance(distance)
                                .set_heart_rate_avg(get_spin_button_value_if_datapoint(
                                    &self_.heart_rate_average_spin_button,
                                    &selected_activity,
                                    ActivityDataPoints::HEART_RATE,
                                ))
                                .set_heart_rate_min(get_spin_button_value_if_datapoint(
                                    &self_.heart_rate_min_spin_button,
                                    &selected_activity,
                                    ActivityDataPoints::HEART_RATE,
                                ))
                                .set_heart_rate_max(get_spin_button_value_if_datapoint(
                                    &self_.heart_rate_max_spin_button,
                                    &selected_activity,
                                    ActivityDataPoints::HEART_RATE,
                                ))
                                .set_steps(get_spin_button_value_if_datapoint(
                                    &self_.steps_spin_button,
                                    &selected_activity,
                                    ActivityDataPoints::STEP_COUNT,
                                ))
                                .set_duration(Duration::minutes(
                                    get_spin_button_value_if_datapoint(
                                        &self_.calories_burned_spin_button,
                                        &selected_activity,
                                        ActivityDataPoints::DURATION,
                                    )
                                    .unwrap_or(0)
                                    .into(),
                                ));

                            if let Err(e) = self_
                                .inner
                                .borrow()
                                .database
                                .as_ref()
                                .unwrap()
                                .save_activity(activity)
                                .await
                            {
                                glib::g_warning!(
                                    crate::config::LOG_DOMAIN,
                                    "Failed to save new data due to error {}",
                                    e.to_string()
                                )
                            }
                            self_.save_recent_activity();

                            obj.destroy();
                        }
                    });
                }
                _ => {
                    obj.destroy();
                }
            });

            self.calories_burned_spin_button.connect_changed(clone!(@weak obj => move |_| {
                let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                {
                    let activity = &self_.inner.borrow_mut().activity;
                    activity.set_calories_burned(Some(get_spinbutton_value(&self_.calories_burned_spin_button)));
                    activity.autofill_from_calories();
                }
                self_.set_spin_buttons_from_activity(self_.calories_burned_spin_button.upcast_ref());
            }));
            self.distance_action_row
                .connect_changed(clone!(@weak obj => move || {
                    let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                    {
                        let activity = &self_.inner.borrow_mut().activity;
                        activity.set_distance(Some(self_.distance_action_row.get_value()));
                        activity.autofill_from_distance();
                    }
                    self_.set_spin_buttons_from_activity(self_.distance_action_row.upcast_ref());
                }));
            self.duration_spin_button.connect_changed(clone!(@weak obj => move |_| {
                let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                {
                    let activity = &self_.inner.borrow_mut().activity;
                    activity.set_duration(Duration::minutes(get_spinbutton_value(&self_.duration_spin_button)));
                    activity.autofill_from_minutes();
                }
                self_.set_spin_buttons_from_activity(self_.duration_spin_button.upcast_ref());
            }));
            self.steps_spin_button
                .connect_changed(clone!(@weak obj => move |_| {
                    let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                    {
                        let activity = &self_.inner.borrow_mut().activity;
                        activity.set_steps(Some(get_spinbutton_value(&self_.steps_spin_button)));
                        activity.autofill_from_steps();
                    }
                    self_.set_spin_buttons_from_activity(self_.steps_spin_button.upcast_ref());
                }));

            self.activity_type_selector
                .connect_activity_selected(clone!(@weak obj => move || {
                    let self_ = imp::HealthActivityAddDialog::from_instance(&obj);
                    self_.set_selected_activity(self_.activity_type_selector.get_selected_activity());
                    let inner = self_.inner.borrow_mut();
                    inner.activity.set_activity_type(inner.selected_activity.activity_type.clone());

                    if let Some(model ) = &inner.filter_model {
                        model.get_filter().map(|f| f.changed(gtk::FilterChange::Different));
                    }
                }));

            self.calories_burned_spin_button
                .connect_input(clone!(@weak obj => move |_| {
                    imp::HealthActivityAddDialog::from_instance(&obj).inner.borrow_mut().calories_burned_spin_button_user_changed = true;
                    None
                }));
            self.distance_action_row.connect_input(clone!(@weak obj => move || {
                imp::HealthActivityAddDialog::from_instance(&obj).inner.borrow_mut().distance_spin_button_user_changed = true;
            }));
            self.duration_spin_button
                .connect_input(clone!(@weak obj => move |_| {
                    imp::HealthActivityAddDialog::from_instance(&obj).inner.borrow_mut().duration_spin_button_user_changed = true;
                    None
                }));
            self.steps_spin_button
                .connect_input(clone!(@weak obj => move |_| {
                    imp::HealthActivityAddDialog::from_instance(&obj).inner.borrow_mut().steps_spin_button_user_changed = true;
                    None
                }));
        }

        fn set_selected_activity(&self, val: ActivityInfo) {
            self.activity_type_menu_button.set_label(&val.name);
        }

        fn filter_activity_entry(&self, o: &glib::Object) -> bool {
            let datapoints = self
                .activity_type_selector
                .get_selected_activity()
                .available_data_points;

            if let Some(row) = o.downcast_ref::<adw::ActionRow>() {
                if (row == &self.activity_type_actionrow.get()
                    || row == &self.date_selector_actionrow.get())
                    || (row == &self.calories_burned_action_row.get()
                        && datapoints.contains(ActivityDataPoints::CALORIES_BURNED))
                    || (row == &self.distance_action_row.get()
                        && datapoints.contains(ActivityDataPoints::DISTANCE))
                    || (row == &self.duration_action_row.get()
                        && datapoints.contains(ActivityDataPoints::DURATION))
                    || (row == &self.stepcount_action_row.get()
                        && datapoints.contains(ActivityDataPoints::STEP_COUNT))
                    || ((row == &self.heart_rate_average_action_row.get()
                        || row == &self.heart_rate_max_action_row.get()
                        || row == &self.heart_rate_min_action_row.get())
                        && datapoints.contains(ActivityDataPoints::HEART_RATE))
                {
                    return true;
                }
            }

            false
        }

        fn save_recent_activity(&self) {
            let inner = self.inner.borrow();

            let mut recent_activities = self.settings.get_recent_activity_types();
            if recent_activities
                .iter()
                .find(|s| &inner.selected_activity.id == s)
                .is_none()
            {
                recent_activities.push(inner.selected_activity.id.to_string());
                if recent_activities.len() > 4 {
                    self.settings.set_recent_activity_types(
                        &recent_activities[1..recent_activities.len()]
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>(),
                    );
                } else {
                    self.settings.set_recent_activity_types(
                        &recent_activities
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>(),
                    );
                }
            }
        }

        fn set_spin_buttons_from_activity(&self, emitter: &gtk::Widget) {
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
                let mut inner = self.inner.borrow_mut();
                if inner.stop_update {
                    return;
                }

                inner.stop_update = true;

                (
                    inner.activity.get_calories_burned().unwrap_or(0),
                    inner.calories_burned_spin_button_user_changed,
                    inner.activity.get_distance(),
                    inner.distance_spin_button_user_changed,
                    inner.activity.get_duration().num_minutes(),
                    inner.duration_spin_button_user_changed,
                    inner.activity.get_steps().unwrap_or(0),
                    inner.steps_spin_button_user_changed,
                )
            };

            if calories != 0
                && calories != get_spinbutton_value(&self.calories_burned_spin_button)
                && self
                    .calories_burned_action_row
                    .get()
                    .upcast_ref::<gtk::Widget>()
                    != emitter
                && !calories_changed
            {
                self.calories_burned_spin_button.set_value(calories.into());
            }
            #[allow(clippy::unnecessary_unwrap)]
            if distance.is_some()
                && distance != Some(self.distance_action_row.get_value())
                && self.distance_action_row.get().upcast_ref::<gtk::Widget>() != emitter
                && !distance_changed
            {
                self.distance_action_row.set_value(distance.unwrap());
            }
            if minutes != 0
                && minutes != get_spinbutton_value::<i64>(&self.duration_spin_button)
                && self.duration_action_row.get().upcast_ref::<gtk::Widget>() != emitter
                && !minutes_changed
            {
                self.duration_spin_button.set_value(minutes as f64);
            }
            if steps != 0
                && steps != get_spinbutton_value(&self.steps_spin_button)
                && self.stepcount_action_row.get().upcast_ref::<gtk::Widget>() != emitter
                && !steps_changed
            {
                self.steps_spin_button.set_value(steps.into());
            }

            self.inner.borrow_mut().stop_update = false;
        }
    }
}

glib::wrapper! {
    pub struct HealthActivityAddDialog(ObjectSubclass<imp::HealthActivityAddDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog;
}

impl HealthActivityAddDialog {
    pub fn new(database: HealthDatabase, parent: &gtk::Window) -> Self {
        let o: HealthActivityAddDialog = glib::Object::new(&[("use-header-bar", &1)])
            .expect("Failed to create HealthActivityAddDialog");

        o.set_transient_for(Some(parent));
        imp::HealthActivityAddDialog::from_instance(&o).set_database(database);

        o
    }
}
