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
    core::ni18n_f,
    model::ActivityType,
    plugins::{
        calories::{GraphModelCalories, GraphModelCaloriesMocked},
        PluginDetails,
    },
    prelude::*,
    views::{BarGraphView, SplitBar},
};
use crate::{model::ActivityInfo, widgets::LegendRow};
use gtk::{
    glib::{self, subclass::prelude::*, Boxed, Cast},
    prelude::*,
};
use std::{cell::RefCell, convert::TryInto, rc::Rc};

mod imp {
    use super::{DataProvider, DataProviderBoxed};
    use crate::{
        core::Database, plugins::PluginDetails, prelude::*, views::BarGraphView, widgets::LegendRow,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{self, Cast},
        CompositeTemplate,
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
        pub calories_graph_model: RefCell<Option<DataProvider>>,
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
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginCaloriesDetails {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            Database::instance().connect_activities_updated(glib::clone!(@weak obj => move |_| {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));

            Database::instance().connect_weights_updated(glib::clone!(@weak obj => move |_| {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<DataProviderBoxed>("data-provider")
                        .construct()
                        .write_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "data-provider" => {
                    self.calories_graph_model.replace(Some(
                        value
                            .get::<DataProviderBoxed>()
                            .unwrap()
                            .0
                            .borrow_mut()
                            .take()
                            .unwrap(),
                    ));
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for PluginCaloriesDetails {}
    impl BinImpl for PluginCaloriesDetails {}
    impl PluginDetailsImpl for PluginCaloriesDetails {
        fn update(&self, obj: &PluginDetails) -> PinnedResultFuture<()> {
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
    /// An implementation of [PluginDetails] that visualizes calorie spent records.
    pub struct PluginCaloriesDetails(ObjectSubclass<imp::PluginCaloriesDetails>)
        @extends gtk::Widget, adw::Bin, PluginDetails,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginCaloriesDetails {
    /// Create a new [PluginCaloriesDetails] to display previous calorie data.
    pub fn new(data_provider: DataProvider) -> Self {
        glib::Object::builder()
            .property(
                "is-mocked",
                matches!(data_provider, DataProvider::Mocked(_)),
            )
            .property(
                "data-provider",
                &DataProviderBoxed(Rc::new(RefCell::new(Some(data_provider)))),
            )
            .build()
    }

    /// Reload the [GraphModelcalories](crate::plugins::calories::GraphModelCalories)'s data and refresh labels & the [BarGraphView](crate::views::BarGraphView).
    pub async fn update(&self) {
        let imp = self.imp();

        let mut calories_graph_model = { imp.calories_graph_model.borrow_mut().take().unwrap() };
        if let Err(e) = calories_graph_model
            .reload(glib::TimeSpan::from_days(30))
            .await
        {
            glib::g_warning!(crate::config::LOG_DOMAIN, "Failed to reload step data: {e}",);
        }

        let distinct_activities = calories_graph_model.distinct_activities();
        for i in 0..3 {
            if i < distinct_activities.len() {
                let info = ActivityInfo::from(distinct_activities[i]);
                let legend_row = imp
                    .legend_box
                    .child_at(0, i as i32)
                    .unwrap()
                    .downcast::<LegendRow>()
                    .unwrap();
                legend_row.set_color(info.color);
                legend_row.set_activity_name(&info.name);
            }
            imp.legend_box
                .child_at(0, i as i32)
                .unwrap()
                .set_visible(i < distinct_activities.len());
        }

        if let Some(view) = imp.calories_graph_view.get() {
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

            imp.scrolled_window.set_child(Some(&calories_graph_view));
            self.switch_to_data_page();
        }

        imp.calories_graph_model.replace(Some(calories_graph_model));
    }
}

#[derive(Clone, Boxed)]
#[boxed_type(name = "HealthDataProviderCaloriesBoxed")]
pub struct DataProviderBoxed(Rc<RefCell<Option<DataProvider>>>);

#[derive(Debug)]
pub enum DataProvider {
    Actual(GraphModelCalories),
    Mocked(GraphModelCaloriesMocked),
}

impl DataProvider {
    pub fn actual() -> Self {
        Self::Actual(GraphModelCalories::new())
    }

    pub fn mocked() -> Self {
        Self::Mocked(GraphModelCaloriesMocked::new())
    }

    pub async fn reload(&mut self, duration: glib::TimeSpan) -> anyhow::Result<()> {
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

#[cfg(test)]
mod test {
    use super::{DataProvider, PluginCaloriesDetails};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginCaloriesDetails::new(DataProvider::mocked());
    }
}
