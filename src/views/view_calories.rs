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

use crate::{
    core::{i18n_f, Database},
    views::{BarGraphView, View},
};
use crate::{model::ActivityInfo, widgets::LegendRow};
use chrono::Duration;
use gtk::glib::{self, subclass::prelude::*, Cast};
use gtk::prelude::GridExt;
use gtk::prelude::WidgetExt;
mod imp {
    use crate::{
        model::GraphModelCalories,
        views::{BarGraphView, View},
        widgets::LegendRow,
    };
    use gtk::{
        glib::{self, Cast},
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/calorie_view.ui")]
    pub struct ViewCalories {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub legend_rows: TemplateChild<LegendRow>,
        #[template_child]
        pub legend_box: TemplateChild<gtk::Grid>,
        pub calories_graph_view: OnceCell<BarGraphView>,
        pub calories_graph_model: RefCell<GraphModelCalories>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewCalories {
        const NAME: &'static str = "HealthViewCalories";
        type ParentType = View;
        type Type = super::ViewCalories;

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

    impl WidgetImpl for ViewCalories {}

    impl ObjectImpl for ViewCalories {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes calorie Spent records.
    pub struct ViewCalories(ObjectSubclass<imp::ViewCalories>)
        @extends View;
}

impl ViewCalories {
    /// Create a new [ViewCalories] to display previous calorie data.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewCalories");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    /// Reload the [GraphModelcalories]'s data and refresh labels & the [GraphView].
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

        let view = self.upcast_ref::<View>();
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
                i18n_f(
                    "{}:\n{} calories\n{}",
                    &[&p.activity_name, &p.calories.to_string(), &p.message],
                )
            })));

            self_.scrolled_window.set_child(Some(&calories_graph_view));
            view.stack().set_visible_child_name("data_page");
        }

        self_.calories_graph_model.replace(calories_graph_model);
    }

    fn imp(&self) -> &imp::ViewCalories {
        imp::ViewCalories::from_instance(self)
    }
}
