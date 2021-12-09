/* view_calories.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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

use std::convert::TryInto;

use crate::{
    core::Database,
    ni18n_f,
    views::{BarGraphView, View},
    ViewExt,
};
use crate::{model::ActivityInfo, widgets::LegendRow};
use chrono::Duration;
use gtk::{
    glib::{self, subclass::prelude::*, Cast},
    prelude::*,
};

mod imp {
    use crate::{
        plugins::calories::GraphModelCalories,
        views::{BarGraphView, PinnedResultFuture, View, ViewImpl},
        widgets::LegendRow,
    };
    use gtk::{
        gio,
        glib::{self, Cast},
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/calories/details.ui")]
    pub struct PluginCaloriesDetails {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub legend_box: TemplateChild<gtk::Grid>,
        pub calories_graph_view: OnceCell<BarGraphView>,
        pub calories_graph_model: RefCell<GraphModelCalories>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginCaloriesDetails {
        const NAME: &'static str = "HealthPluginCaloriesDetails";
        type ParentType = View;
        type Type = super::PluginCaloriesDetails;

        fn class_init(klass: &mut Self::Class) {
            LegendRow::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<View>().init_template();
            }
        }
    }

    impl WidgetImpl for PluginCaloriesDetails {}

    impl ObjectImpl for PluginCaloriesDetails {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ViewImpl for PluginCaloriesDetails {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginCaloriesDetails>()
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
    /// An implementation of [View] visualizes calorie Spent records.
    pub struct PluginCaloriesDetails(ObjectSubclass<imp::PluginCaloriesDetails>)
        @extends gtk::Widget, View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginCaloriesDetails {
    /// Create a new [PluginCaloriesDetails] to display previous calorie data.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create PluginCaloriesDetails");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        Database::instance().connect_weights_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    /// Reload the [GraphModelcalories](crate::model::GraphModelCalories)'s data and refresh labels & the [BarGraphView](crate::views::BarGraphView).
    pub async fn update(&self) {
        let self_ = self.imp();

        let mut calories_graph_model = { self_.calories_graph_model.borrow().clone() };
        if let Err(e) = calories_graph_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload step data: {}",
                e
            );
        }

        let distinct_activities = calories_graph_model.distinct_activities.clone();
        for i in 0..3 {
            if i < distinct_activities.len() {
                self_
                    .legend_box
                    .child_at(0, i as i32)
                    .unwrap()
                    .downcast::<LegendRow>()
                    .unwrap()
                    .set_legend_row(
                        ActivityInfo::from(distinct_activities[i].clone()).color,
                        ActivityInfo::from(distinct_activities[i].clone()).name,
                    );
            }
            self_
                .legend_box
                .child_at(0, i as i32)
                .unwrap()
                .set_visible(i < distinct_activities.len());
        }

        if let Some(view) = self_.calories_graph_view.get() {
            view.set_split_bars(calories_graph_model.to_split_bar());
        } else if !calories_graph_model.is_empty() {
            let calories_graph_view = BarGraphView::new();
            calories_graph_view.set_rmr(calories_graph_model.rmr);
            calories_graph_view.set_split_bars(calories_graph_model.to_split_bar());
            calories_graph_view.set_x_lines_interval(100.0);
            calories_graph_view.set_rmr(calories_graph_model.rmr);
            calories_graph_view.set_hover_func(Some(Box::new(|p| {
                ni18n_f(
                    "{}:\n{} calorie\n{}",
                    "{}:\n{} calories\n{}",
                    p.calories.try_into().unwrap_or(0),
                    &[&p.activity_name, &p.calories.to_string(), &p.message],
                )
            })));

            self_.scrolled_window.set_child(Some(&calories_graph_view));
            self.stack().set_visible_child_name("data_page");
        }

        self_.calories_graph_model.replace(calories_graph_model);
    }

    fn imp(&self) -> &imp::PluginCaloriesDetails {
        imp::PluginCaloriesDetails::from_instance(self)
    }
}
