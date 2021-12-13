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
    plugins::{
        calories::{GraphModelCalories, GraphModelCaloriesMocked},
        PluginDetails, PluginDetailsExt,
    },
    views::BarGraphView,
};
use crate::{model::ActivityInfo, widgets::LegendRow};
use chrono::Duration;
use gtk::{
    glib::{self, subclass::prelude::*, Cast},
    prelude::*,
};
use gtk_macros::spawn;

use self::imp::DataProvider;

mod imp {
    use crate::{
        plugins::{
            calories::{GraphModelCalories, GraphModelCaloriesMocked},
            PluginDetails, PluginDetailsImpl,
        },
        views::{BarGraphView, PinnedResultFuture},
        widgets::LegendRow,
        ActivityType, SplitBar,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{self, Cast},
        {subclass::prelude::*, CompositeTemplate},
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, Clone)]
    pub enum DataProvider {
        Actual(GraphModelCalories),
        Mocked(GraphModelCaloriesMocked),
    }

    impl Default for DataProvider {
        fn default() -> Self {
            Self::Actual(GraphModelCalories::default())
        }
    }

    impl DataProvider {
        pub async fn reload(&mut self, duration: chrono::Duration) -> anyhow::Result<()> {
            match self {
                Self::Actual(m) => m.reload(duration).await,
                Self::Mocked(m) => m.reload(duration).await,
            }
        }

        /// Converts the model's data to an array of `SplitBars` so it can be displayed in a `BarGraphView`.
        pub fn to_split_bar(&self) -> Vec<SplitBar> {
            match self {
                Self::Actual(m) => m.to_split_bar(),
                Self::Mocked(m) => m.to_split_bar(),
            }
        }

        pub async fn rmr(&self) -> f32 {
            match self {
                Self::Actual(m) => m.rmr().await,
                Self::Mocked(m) => m.rmr().await,
            }
        }

        pub fn is_empty(&self) -> bool {
            match self {
                Self::Actual(m) => m.is_empty(),
                Self::Mocked(m) => m.is_empty(),
            }
        }

        pub fn distinct_activities(&self) -> &[ActivityType] {
            match self {
                Self::Actual(m) => m.distinct_activities(),
                Self::Mocked(m) => m.distinct_activities(),
            }
        }
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/calories/details.ui")]
    pub struct PluginCaloriesDetails {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub legend_box: TemplateChild<gtk::Grid>,
        pub calories_graph_view: OnceCell<BarGraphView>,
        pub calories_graph_model: RefCell<DataProvider>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginCaloriesDetails {
        const NAME: &'static str = "HealthPluginCaloriesDetails";
        type ParentType = PluginDetails;
        type Type = super::PluginCaloriesDetails;

        fn class_init(klass: &mut Self::Class) {
            LegendRow::static_type();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<PluginDetails>().init_template();
            }
        }
    }

    impl ObjectImpl for PluginCaloriesDetails {}
    impl WidgetImpl for PluginCaloriesDetails {}
    impl BinImpl for PluginCaloriesDetails {}
    impl PluginDetailsImpl for PluginCaloriesDetails {
        fn update_actual(&self, obj: &PluginDetails) -> PinnedResultFuture {
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
        @extends gtk::Widget, adw::Bin, PluginDetails,
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

        let distinct_activities = calories_graph_model.distinct_activities();
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
        } else if calories_graph_model.is_empty() {
            self.switch_to_empty_page();
        } else {
            let calories_graph_view = BarGraphView::new();
            calories_graph_view.set_rmr(calories_graph_model.rmr().await);
            calories_graph_view.set_split_bars(calories_graph_model.to_split_bar());
            calories_graph_view.set_x_lines_interval(100.0);
            calories_graph_view.set_rmr(calories_graph_model.rmr().await);
            calories_graph_view.set_hover_func(Some(Box::new(|p| {
                ni18n_f(
                    "{}:\n{} calorie\n{}",
                    "{}:\n{} calories\n{}",
                    p.calories.try_into().unwrap_or(0),
                    &[&p.activity_name, &p.calories.to_string(), &p.message],
                )
            })));

            self_.scrolled_window.set_child(Some(&calories_graph_view));
            self.switch_to_data_page();
        }

        self_.calories_graph_model.replace(calories_graph_model);
    }

    pub fn mock(&self) {
        self.imp()
            .calories_graph_model
            .replace(DataProvider::Mocked(GraphModelCaloriesMocked::default()));
        spawn!(glib::clone!(@weak self as obj => async move {
            obj.update().await;
        }));
    }

    pub fn unmock(&self) {
        self.imp()
            .calories_graph_model
            .replace(DataProvider::Actual(GraphModelCalories::default()));
        spawn!(glib::clone!(@weak self as obj => async move {
            obj.update().await;
        }));
    }

    fn imp(&self) -> &imp::PluginCaloriesDetails {
        imp::PluginCaloriesDetails::from_instance(self)
    }
}
