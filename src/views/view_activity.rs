/* view_activity.rs
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
    core::{date::prelude::DateExt, i18n_f, Database},
    model::ViewPeriod,
    views::View,
};
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*, Cast},
    prelude::*,
};
use gtk_macros::stateful_action;
use std::str::FromStr;

mod imp {
    use crate::{
        core::Settings,
        model::{Activity, ModelActivity, ViewPeriod},
        views::{PinnedResultFuture, View, ViewImpl},
        widgets::ActivityRow,
    };
    use gtk::{
        self, gio,
        glib::{self, Cast},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };

    #[derive(Debug, Default)]
    pub struct ViewActivityMut {
        pub period: ViewPeriod,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_view.ui")]
    pub struct ViewActivity {
        pub inner: std::cell::RefCell<ViewActivityMut>,
        settings: Settings,
        pub activity_model: ModelActivity,
        #[template_child]
        pub activities_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub stack_activity: TemplateChild<gtk::Stack>,
        #[template_child]
        pub toggle_week: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub toggle_month: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub toggle_quarter: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub toggle_year: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub toggle_all: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub since_date: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewActivity {
        const NAME: &'static str = "HealthViewActivity";
        type ParentType = View;
        type Type = super::ViewActivity;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<View>().init_template();
            }
        }
    }

    impl WidgetImpl for ViewActivity {}

    impl ObjectImpl for ViewActivity {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.activities_list_box
                .bind_model(Some(&self.activity_model), |o| {
                    let row = ActivityRow::new();
                    row.set_activity(o.clone().downcast::<Activity>().unwrap());
                    row.upcast()
                });
            obj.setup_actions();
        }
    }

    impl ViewImpl for ViewActivity {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::ViewActivity>()
                            .unwrap()
                            .update()
                            .await;
                        send.resolve(Ok(()));
                    });
                }),
            ))
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct ViewActivity(ObjectSubclass<imp::ViewActivity>)
        @extends View, gtk::Widget;
}

impl ViewActivity {
    /// Create a new [ViewActivity] to display previous activities.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewActivity");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    pub fn set_view_period(&self, view_period: ViewPeriod) {
        let downgraded = self.downgrade();
        gtk_macros::spawn!(async move {
            if let Some(obj) = downgraded.upgrade() {
                obj.imp().inner.borrow_mut().period = view_period;
                obj.update().await;
                obj.imp().toggle_week.set_active(false);
                obj.imp().toggle_month.set_active(false);
                obj.imp().toggle_quarter.set_active(false);
                obj.imp().toggle_year.set_active(false);
                obj.imp().toggle_all.set_active(false);
                match obj.imp().inner.borrow().period {
                    ViewPeriod::Week => obj.imp().toggle_week.set_active(true),
                    ViewPeriod::Month => obj.imp().toggle_month.set_active(true),
                    ViewPeriod::Quarter => obj.imp().toggle_quarter.set_active(true),
                    ViewPeriod::Year => obj.imp().toggle_year.set_active(true),
                    ViewPeriod::All => obj.imp().toggle_all.set_active(true),
                }
            }
        });
    }

    /// Reload the [ModelActivity]'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = &self.imp().activity_model;
        let new_period = self.imp().inner.borrow().period;
        let reload_result = activity_model.reload(new_period).await;
        if let Err(e) = reload_result {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload activity data: {}",
                e
            );
        } else if let Ok(Some(date)) = reload_result {
            self.imp().since_date.set_label(&i18n_f(
                "No activities on or after {}",
                &[&date.format_local()],
            ));
        }

        if activity_model.activity_present().await {
            self.upcast_ref::<View>()
                .stack()
                .set_visible_child_name("data_page");
            self.imp()
                .stack_activity
                .set_visible_child_name(if !activity_model.is_empty() {
                    "recent_activities"
                } else {
                    "no_recent"
                });
        }
    }

    fn handle_view_period(&self, action: &gio::SimpleAction, parameter: Option<&glib::Variant>) {
        let parameter = parameter.unwrap();

        self.set_view_period(
            ViewPeriod::from_str(parameter.get::<String>().unwrap().as_str()).unwrap(),
        );

        action.set_state(parameter);
    }

    fn imp(&self) -> &imp::ViewActivity {
        imp::ViewActivity::from_instance(self)
    }

    fn setup_actions(&self) {
        let action_group = gio::SimpleActionGroup::new();

        stateful_action!(
            action_group,
            "view_period",
            Some(&String::static_variant_type()),
            "week",
            clone!(@weak self as obj => move |a, p| {
                obj.handle_view_period(a, p);
            })
        );

        self.insert_action_group("view_activity", Some(&action_group));
    }
}
