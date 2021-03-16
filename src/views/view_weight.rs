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
    core::{i18n, i18n_f, settings::Unitsystem, utils::round_decimal_places, Database},
    model::GraphModelWeight,
    views::{GraphView, View},
};
use chrono::Duration;
use gio::subclass::prelude::*;
use glib::Cast;
use uom::si::{
    length::meter,
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::Settings,
        model::GraphModelWeight,
        views::{GraphView, View},
    };
    use glib::Cast;
    use gtk::{subclass::prelude::*, CompositeTemplate, WidgetExt};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/weight_view.ui")]
    pub struct ViewWeight {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        pub settings: Settings,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub weight_graph_view: OnceCell<GraphView>,
        pub weight_graph_model: RefCell<GraphModelWeight>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewWeight {
        const NAME: &'static str = "HealthViewWeight";
        type ParentType = View;
        type Type = super::ViewWeight;

        fn new() -> Self {
            Self {
                scrolled_window: TemplateChild::default(),
                settings: Settings::get_instance(),
                settings_handler_id: RefCell::new(None),
                weight_graph_view: OnceCell::new(),
                weight_graph_model: RefCell::new(GraphModelWeight::new()),
            }
        }

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

    impl WidgetImpl for ViewWeight {}

    impl ObjectImpl for ViewWeight {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(id) = self.settings_handler_id.borrow_mut().take() {
                self.settings.disconnect(id);
            }
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes BMI and weight development.
    pub struct ViewWeight(ObjectSubclass<imp::ViewWeight>)
        @extends View;
}

impl ViewWeight {
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewWeight");

        Database::get_instance().connect_weights_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    /// Reload the [GraphModelWeight]'s data and refresh labels & reload the [GraphView].
    pub async fn update(&self) {
        let self_ = self.get_priv();
        let mut weight_graph_model = { self_.weight_graph_model.borrow().clone() };
        if let Err(e) = weight_graph_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload weight data: {}",
                e
            );
        }

        let view = self.upcast_ref::<View>();
        view.set_title(i18n_f(
            "Current BMI: {}",
            &[&self.get_bmi(&weight_graph_model)],
        ));
        self.update_weightgoal_label(&weight_graph_model);

        if let Some(view) = self_.weight_graph_view.get() {
            view.set_points(weight_graph_model.to_points());
        } else if !weight_graph_model.is_empty() {
            let weight_graph_view = GraphView::new();
            weight_graph_view.set_points(weight_graph_model.to_points());
            weight_graph_view.set_x_lines_interval(10.0);
            let settings = self_.settings.clone();
            weight_graph_view.set_hover_func(Some(Box::new(move |p| {
                let unit = if settings.get_unitsystem() == Unitsystem::Imperial {
                    "PB"
                } else {
                    "KG"
                };

                return i18n_f(
                    "{} {} on {}",
                    &[
                        &p.value.to_string(),
                        unit,
                        &format!("{}", p.date.format("%x")),
                    ],
                );
            })));
            let unitgoal = self_.settings.get_user_weightgoal();
            let weightgoal = if self_.settings.get_unitsystem() == Unitsystem::Imperial {
                unitgoal.get::<pound>()
            } else {
                unitgoal.get::<kilogram>()
            };
            weight_graph_view.set_limit(Some(weightgoal));
            weight_graph_view.set_limit_label(Some(i18n("Weightgoal")));

            self_.scrolled_window.set_child(Some(&weight_graph_view));
            view.get_stack().set_visible_child_name("data_page");

            self_.weight_graph_view.set(weight_graph_view).unwrap();

            self_.settings_handler_id.replace(Some(
                self_.settings.connect_user_weightgoal_changed(
                    glib::clone!(@weak self as obj => move |_,_| {
                        glib::MainContext::default().spawn_local(async move {
                            obj.update().await
                        })
                    }),
                ),
            ));
        }

        self_.weight_graph_model.replace(weight_graph_model);
    }

    fn get_bmi(&self, model: &GraphModelWeight) -> String {
        if let Some(last_weight) = model.get_last_weight() {
            let height = self.get_priv().settings.get_user_height().get::<meter>() as f32;
            let bmi =
                round_decimal_places(last_weight.get::<kilogram>() as f32 / (height * height), 1);
            format!("{bmi:.1}", bmi = bmi)
        } else {
            i18n("Unknown BMI")
        }
    }

    fn get_priv(&self) -> &imp::ViewWeight {
        imp::ViewWeight::from_instance(self)
    }

    fn update_weightgoal_label(&self, model: &GraphModelWeight) {
        let self_ = self.get_priv();
        let weightgoal = self_.settings.get_user_weightgoal();
        let unitsystem = self_.settings.get_unitsystem();
        let (weight_value, translation) = if unitsystem == Unitsystem::Imperial {
            (weightgoal.get::<pound>(), i18n("pounds"))
        } else {
            (weightgoal.get::<kilogram>(), i18n("kilogram"))
        };
        let goal_label = self.upcast_ref::<View>().get_goal_label();

        if weight_value > 0.1 && model.is_empty() {
            /* TRANSLATORS: the second {} format strings is the weight unit, e.g. kilogram */
            goal_label.set_text (&i18n_f("Your weight goal is {} {}. Add a first weight measurement to see how close you are to reaching it.",&[&weight_value.to_string(), &translation]));
        } else if weight_value > 0.1 && !model.is_empty() {
            if model.get_last_weight().unwrap() == weightgoal {
                goal_label.set_text(&i18n("You've reached your weightgoal. Great job!"));
            }

            let unit_diff = model.get_last_weight().unwrap() - weightgoal;
            let mut diff = if unitsystem == Unitsystem::Imperial {
                unit_diff.get::<pound>()
            } else {
                unit_diff.get::<kilogram>()
            };

            if diff < 0.0 {
                diff *= -1.0;
            }

            /* TRANSLATORS: the second & fourth {} format strings is the weight unit, e.g. kilogram */
            goal_label.set_text(&i18n_f(
                "{} {} left to reach your weightgoal of {} {}",
                &[
                    &format!("{diff:.1}", diff = round_decimal_places(diff, 1)),
                    &translation,
                    &format!("{weight_value:.1}", weight_value = weight_value),
                    &translation,
                ],
            ));
        } else {
            goal_label.set_text(&i18n(
                "No weightgoal set yet. You can set it in Health's preferences.",
            ));
        }
    }
}
