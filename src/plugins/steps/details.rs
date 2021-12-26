/* view.rs
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
    core::{i18n, i18n_f, ni18n_f},
    plugins::{
        steps::{GraphModelSteps, GraphModelStepsMocked},
        PluginDetails,
    },
    prelude::*,
    views::{GraphView, Point},
};
use chrono::Duration;
use gtk::glib::{self, subclass::prelude::*, Boxed};
use std::{cell::RefCell, rc::Rc};

mod imp {
    use super::{DataProvider, DataProviderBoxed};
    use crate::{
        core::{Database, Settings},
        plugins::PluginDetails,
        prelude::*,
        views::GraphView,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        gio,
        glib::{self, Cast},
        {subclass::prelude::*, CompositeTemplate},
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/steps/details.ui")]
    pub struct PluginStepsDetails {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        pub settings: Settings,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub steps_graph_view: OnceCell<GraphView>,
        pub steps_graph_model: RefCell<Option<DataProvider>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginStepsDetails {
        const NAME: &'static str = "HealthPluginStepsDetails";
        type ParentType = PluginDetails;
        type Type = super::PluginStepsDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginStepsDetails {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            Database::instance().connect_activities_updated(glib::clone!(@weak obj => move || {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(id) = self.settings_handler_id.borrow_mut().take() {
                self.settings.disconnect(id);
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "data-provider",
                    "data-provider",
                    "data-provider",
                    DataProviderBoxed::static_type(),
                    glib::ParamFlags::CONSTRUCT | glib::ParamFlags::WRITABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "data-provider" => {
                    self.steps_graph_model.replace(Some(
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
    impl WidgetImpl for PluginStepsDetails {}
    impl BinImpl for PluginStepsDetails {}
    impl PluginDetailsImpl for PluginStepsDetails {
        fn update(&self, obj: &PluginDetails) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginStepsDetails>()
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
    /// An implementation of [PluginDetails] that visualizes streak counts and daily step records.
    pub struct PluginStepsDetails(ObjectSubclass<imp::PluginStepsDetails>)
        @extends gtk::Widget, adw::Bin, PluginDetails,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginStepsDetails {
    /// Create a new [PluginStepsDetails] to display previous step activity.
    pub fn new(data_provider: DataProvider) -> Self {
        glib::Object::new(&[
            (
                "is-mocked",
                &matches!(data_provider, DataProvider::Mocked(_)),
            ),
            (
                "data-provider",
                &DataProviderBoxed(Rc::new(RefCell::new(Some(data_provider)))),
            ),
        ])
        .expect("Failed to create PluginStepsDetails")
    }

    // TRANSLATORS notes have to be on the same line, so we cant split them
    #[rustfmt::skip]
    /// Reload the [GraphModelSteps](crate::plugins::steps::GraphModelSteps)'s data and refresh labels & the [GraphView].
    pub async fn update(&self) {
        let self_ = self.imp();

        let mut steps_graph_model = { self_.steps_graph_model.borrow_mut().take().unwrap() };
        if let Err(e) = steps_graph_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload step data: {}",
                e
            );
        }

        self.set_filled_title(&i18n_f(
            "Today's steps: {}",
            &[&steps_graph_model
                .today_step_count()
                .unwrap_or(0)
                .to_string()],
        ));

        let streak_count = steps_graph_model.streak_count_today(self_.settings.user_step_goal());

        match streak_count {
            0 => {
                let previous_streak =
                    steps_graph_model.streak_count_yesterday(self_.settings.user_step_goal());
                if previous_streak == 0 {
                    self.set_filled_subtitle(&i18n(
                        "No streak yet. Reach your step goal for multiple days to start a streak!",
                    ));
                } else {
                    self.set_filled_subtitle(&ni18n_f(
                        "You're on a streak for {} day. Reach your step goal today to continue it!",
                        "You're on a streak for {} days. Reach your step goal today to continue it!",
                        previous_streak,
                        &[&previous_streak.to_string()],
                    ));
                }
            }
            1 => self.set_filled_subtitle(&i18n(
                "You've reached your step goal today. Keep going to start a streak!",
            )),
            _ => self.set_filled_subtitle(&ni18n_f(
                "You're on a streak for {} day. Good job!",
                "You're on a streak for {} days. Good job!",
                streak_count,
                &[&streak_count.to_string()],
            )),
        }

        if let Some(view) = self_.steps_graph_view.get() {
            view.set_points(steps_graph_model.to_points());
        } else if steps_graph_model.is_empty() {
            self.switch_to_empty_page();
        } else {
            let steps_graph_view = GraphView::new();
            steps_graph_view.set_points(steps_graph_model.to_points());
            steps_graph_view.set_x_lines_interval(500.0);
            steps_graph_view.set_hover_func(Some(Box::new(|p| {
                // TRANSLATORS: X step(s) on DATE
                ni18n_f( "{} step on {}",
                    "{} steps on {}",
                    p.value as u32,
                    &[&p.value.to_string(), &p.date.format_local()],
                )
            })));
            steps_graph_view.set_limit(Some(self_.settings.user_step_goal() as f32));
            steps_graph_view.set_limit_label(Some(i18n("Step goal")));

            self_.scrolled_window.set_child(Some(&steps_graph_view));
            self.switch_to_data_page();

            self_.steps_graph_view.set(steps_graph_view).unwrap();

            self_
                .settings_handler_id
                .replace(Some(self_.settings.connect_user_step_goal_changed(
                    glib::clone!(@weak self as obj => move |_,_| {
                        gtk_macros::spawn!(async move {
                            obj.update().await;
                        });
                    }),
                )));
        }

        self_.steps_graph_model.replace(Some(steps_graph_model));
    }

    fn imp(&self) -> &imp::PluginStepsDetails {
        imp::PluginStepsDetails::from_instance(self)
    }
}

#[derive(Clone, Boxed)]
#[boxed_type(name = "HealthDataProviderStepssBoxed")]
pub struct DataProviderBoxed(Rc<RefCell<Option<DataProvider>>>);

#[derive(Debug)]
pub enum DataProvider {
    Actual(GraphModelSteps),
    Mocked(GraphModelStepsMocked),
}

impl DataProvider {
    pub fn actual() -> Self {
        Self::Actual(GraphModelSteps::new())
    }

    pub fn mocked() -> Self {
        Self::Mocked(GraphModelStepsMocked::new())
    }

    pub fn today_step_count(&self) -> Option<u32> {
        match self {
            Self::Actual(m) => m.today_step_count(),
            Self::Mocked(m) => m.today_step_count(),
        }
    }
    pub fn streak_count_today(&self, step_goal: u32) -> u32 {
        match self {
            Self::Actual(m) => m.streak_count_today(step_goal),
            Self::Mocked(m) => m.streak_count_today(step_goal),
        }
    }
    pub fn streak_count_yesterday(&self, step_goal: u32) -> u32 {
        match self {
            Self::Actual(m) => m.streak_count_yesterday(step_goal),
            Self::Mocked(m) => m.streak_count_yesterday(step_goal),
        }
    }
    pub async fn reload(&mut self, duration: Duration) -> anyhow::Result<()> {
        match self {
            Self::Actual(m) => m.reload(duration).await,
            Self::Mocked(m) => m.reload(duration).await,
        }
    }
    pub fn to_points(&self) -> Vec<Point> {
        match self {
            Self::Actual(m) => m.to_points(),
            Self::Mocked(m) => m.to_points(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Actual(m) => m.is_empty(),
            Self::Mocked(m) => m.is_empty(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DataProvider, PluginStepsDetails};
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginStepsDetails::new(DataProvider::mocked());
    }
}
