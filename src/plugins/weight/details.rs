/* view_weight.rs
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
    core::{i18n, i18n_f, ni18n_f, UnitSystem},
    model::User,
    plugins::{
        weight::{GraphModelWeight, GraphModelWeightMocked},
        PluginDetails,
    },
    prelude::*,
    views::GraphView,
};
use gtk::{
    gio::subclass::prelude::*,
    glib::{self, Boxed},
};
use std::{cell::RefCell, rc::Rc};
use uom::si::{
    f32::Mass,
    length::meter,
    mass::{kilogram, pound},
};

mod imp {
    use super::{DataProvider, DataProviderBoxed};
    use crate::{
        core::{Database, Settings},
        plugins::PluginDetails,
        prelude::*,
        views::GraphView,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{gio, glib, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/weight/details.ui")]
    pub struct PluginWeightDetails {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        pub settings: Settings,
        pub database: Database,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub weight_graph_view: OnceCell<GraphView>,
        pub weight_graph_model: RefCell<Option<DataProvider>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginWeightDetails {
        const NAME: &'static str = "HealthPluginWeightDetails";
        type ParentType = PluginDetails;
        type Type = super::PluginWeightDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginWeightDetails {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            Database::instance().connect_weights_updated(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    gtk_macros::spawn!(async move {
                        obj.update().await;
                    });
                }
            ));
        }

        fn dispose(&self) {
            if let Some(id) = self.settings_handler_id.borrow_mut().take() {
                self.settings.disconnect(id);
            }
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
                    self.weight_graph_model.replace(Some(
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
    impl WidgetImpl for PluginWeightDetails {}
    impl NavigationPageImpl for PluginWeightDetails {}
    impl PluginDetailsImpl for PluginWeightDetails {
        fn update(&self, obj: &PluginDetails) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _, send| {
                        gtk_macros::spawn!(async move {
                            obj.downcast_ref::<super::PluginWeightDetails>()
                                .unwrap()
                                .update()
                                .await;
                            send.resolve(Ok(()));
                        });
                    }
                ),
            ))
        }
    }
}

glib::wrapper! {
    /// An implementation of [PluginDetails] that visualizes BMI and weight development.
    pub struct PluginWeightDetails(ObjectSubclass<imp::PluginWeightDetails>)
        @extends gtk::Widget, adw::NavigationPage, PluginDetails,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginWeightDetails {
    /// Create a new [PluginWeightDetails] to display previous weight measurements.
    pub fn new(data_provider: DataProvider) -> Self {
        glib::Object::builder()
            .property(
                "is-mocked",
                matches!(data_provider, DataProvider::Mocked(_)),
            )
            .property(
                "data-provider",
                DataProviderBoxed(Rc::new(RefCell::new(Some(data_provider)))),
            )
            .build()
    }

    pub async fn get_user(&self) -> User {
        let imp = self.imp();
        let user_id = i64::from(imp.settings.active_user_id());
        let user = &imp.database.user(user_id).await.unwrap();
        user.clone()
    }

    // TRANSLATORS notes have to be on the same line, so we cant split them
    #[rustfmt::skip]
    /// Reload the [GraphModelWeight]'s data and refresh labels & reload the [GraphView].
    pub async fn update(&self) {
        let imp = self.imp();
        let user = self.get_user().await;
        let mut weight_graph_model = { imp.weight_graph_model.borrow_mut().take().unwrap() };
        if let Err(e) = weight_graph_model.reload(glib::TimeSpan::from_days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload weight data: {e}",
            );
        }

        self.set_filled_title(&i18n_f("Current BMI: {}", &[&self.bmi(&weight_graph_model).await]));
        self.update_weight_goal_label(&weight_graph_model).await;

        if let Some(view) = imp.weight_graph_view.get() {
            view.set_points(weight_graph_model.to_points());
        } else if weight_graph_model.is_empty() {
            self.switch_to_empty_page();
        } else {
            let weight_graph_view = GraphView::new();
            weight_graph_view.set_points(weight_graph_model.to_points());
            weight_graph_view.set_x_lines_interval(10.0);
            let settings = imp.settings.clone();
            weight_graph_view.set_hover_func(Some(Box::new(move |p| {
                if settings.unit_system() == UnitSystem::Imperial {
                    // TRANSLATORS: Weight X on date Y
                    ni18n_f("{} pound on {}",  "{} pounds on {}", p.value as u32, &[&p.value.to_string(), &p.date.format_local()])
                } else {
                    // TRANSLATORS: Weight X on date Y
                    ni18n_f("{} kilogram on {}", "{} kilograms on {}", p.value as u32, &[&p.value.to_string(), &p.date.format_local()])
                }
            })));
            if let Some(unit_goal) = user.user_weightgoal() {
                let weight_goal = if imp.settings.unit_system() == UnitSystem::Imperial {
                    unit_goal.get::<pound>()
                } else {
                    unit_goal.get::<kilogram>()
                };
                weight_graph_view.set_limit(Some(weight_goal));
                weight_graph_view.set_limit_label(Some(i18n("Weight goal")));
            }

            imp.scrolled_window.set_child(Some(&weight_graph_view));
            self.switch_to_data_page();

            imp.weight_graph_view.set(weight_graph_view).unwrap();

            imp.settings_handler_id.replace(Some(
                imp.database.connect_user_updated(
                    glib::clone!(
                        #[weak(rename_to = obj)]
                        self,
                        move |_| {
                            gtk_macros::spawn!(async move {
                                obj.update().await;
                            });
                        }
                    ),
                ),
            ));
        }

        imp.weight_graph_model.replace(Some(weight_graph_model));
    }

    async fn bmi(&self, model: &DataProvider) -> String {
        let user = self.get_user().await;
        if let Some(last_weight) = model.last_weight() {
            let height = user.user_height().unwrap().get::<meter>();
            if height == 0.0 || last_weight.get::<kilogram>() == 0.0 {
                return i18n("Unknown BMI");
            }
            let bmi = (last_weight.get::<kilogram>() / (height * height)).round_decimal_places(1);
            format!("{bmi:.1}")
        } else {
            i18n("Unknown BMI")
        }
    }

    async fn update_weight_goal_label(&self, model: &DataProvider) {
        let imp = self.imp();
        let user = self.get_user().await;
        if let Some(weight_goal) = user.user_weightgoal() {
            let unit_system = imp.settings.unit_system();
            let weight_value = if unit_system == UnitSystem::Imperial {
                weight_goal.get::<pound>()
            } else {
                weight_goal.get::<kilogram>()
            };

            if weight_value > 0.1 && model.is_empty() {
                let goal_label_text = if unit_system == UnitSystem::Imperial {
                    ni18n_f(
                        "Your weight goal is {} pound. Add a first weight measurement to see how close you are to reaching it.",
                        "Your weight goal is {} pounds. Add a first weight measurement to see how close you are to reaching it.",
                        weight_value as u32,
                        &[&weight_value.to_string()],
                    )
                } else {
                    ni18n_f(
                        "Your weight goal is {} kilogram. Add a first weight measurement to see how close you are to reaching it.",
                        "Your weight goal is {} kilograms. Add a first weight measurement to see how close you are to reaching it.",
                        weight_value as u32,
                        &[&weight_value.to_string()],
                    )
                };
                self.set_filled_subtitle(&goal_label_text);
            } else if weight_value > 0.1 && !model.is_empty() {
                if model.last_weight().unwrap() == weight_goal {
                    self.set_filled_subtitle(&i18n("You’ve reached your weight goal. Great job!"));
                }

                let unit_diff = model.last_weight().unwrap() - weight_goal;
                let mut diff = if unit_system == UnitSystem::Imperial {
                    unit_diff.get::<pound>()
                } else {
                    unit_diff.get::<kilogram>()
                };

                if diff < 0.0 {
                    diff *= -1.0;
                }

                let goal_label_text = if unit_system == UnitSystem::Imperial {
                    ni18n_f(
                        "Your weight goal is {} pound.",
                        "Your weight goal is {} pounds.",
                        weight_value as u32,
                        &[&format!("{weight_value:.1}")],
                    ) + &ni18n_f(
                        "You have {} pound left to reach it.",
                        "You have {} pounds left to reach it.",
                        diff as u32,
                        &[&format!("{diff:.1}", diff = diff.round_decimal_places(1))],
                    )
                } else {
                    ni18n_f(
                        "Your weight goal is {} kilogram.",
                        "Your weight goal is {} kilograms.",
                        weight_value as u32,
                        &[&format!("{weight_value:.1}")],
                    ) + " "
                        + &ni18n_f(
                            "You have {} kilogram left to reach it",
                            "You have {} kilograms left to reach it",
                            diff as u32,
                            &[&format!("{diff:.1}", diff = diff.round_decimal_places(1))],
                        )
                };
                self.set_filled_subtitle(&goal_label_text);
            }
        } else {
            self.set_filled_subtitle(&i18n(
                "No weight goal set yet. You can set it in Health’s preferences.",
            ));
        }
    }
}

#[derive(Clone, Boxed)]
#[boxed_type(name = "HealthDataProviderWeightsBoxed")]
pub struct DataProviderBoxed(Rc<RefCell<Option<DataProvider>>>);

#[derive(Debug)]
pub enum DataProvider {
    Actual(GraphModelWeight),
    Mocked(GraphModelWeightMocked),
}

impl DataProvider {
    pub fn actual() -> Self {
        Self::Actual(GraphModelWeight::new())
    }

    pub fn mocked() -> Self {
        Self::Mocked(GraphModelWeightMocked::new())
    }

    pub async fn reload(&mut self, duration: glib::TimeSpan) -> anyhow::Result<()> {
        match self {
            Self::Actual(m) => m.reload(duration).await,
            Self::Mocked(m) => m.reload(duration).await,
        }
    }

    pub fn to_points(&self) -> Vec<crate::views::Point> {
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

    pub fn last_weight(&self) -> Option<Mass> {
        match self {
            Self::Actual(m) => m.last_weight(),
            Self::Mocked(m) => m.last_weight(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DataProvider, PluginWeightDetails};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginWeightDetails::new(DataProvider::mocked());
    }
}
