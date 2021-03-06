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
    core::{date::prelude::*, i18n, i18n_f, utils::prelude::*, Database, UnitSystem},
    model::GraphModelWeight,
    ni18n_f,
    views::{GraphView, View},
};
use chrono::Duration;
use gtk::{
    gio::subclass::prelude::*,
    glib::{self, Cast},
};
use uom::si::{
    length::meter,
    mass::{kilogram, pound},
};

mod imp {
    use crate::{
        core::Settings,
        model::GraphModelWeight,
        views::{GraphView, PinnedResultFuture, View, ViewImpl},
    };
    use gtk::{
        gio,
        glib::{self, Cast},
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
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

    impl ViewImpl for ViewWeight {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj=> move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::ViewWeight>()
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
    /// An implementation of [View] visualizes BMI and weight development.
    pub struct ViewWeight(ObjectSubclass<imp::ViewWeight>)
        @extends gtk::Widget, View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewWeight {
    /// Create a new [ViewWeight] to display previous weight measurements.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewWeight");

        Database::instance().connect_weights_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }
    // TRANSLATORS notes have to be on the same line, so we cant split them
    #[rustfmt::skip]
    /// Reload the [GraphModelWeight]'s data and refresh labels & reload the [GraphView].
    pub async fn update(&self) {
        let self_ = self.imp();
        let mut weight_graph_model = { self_.weight_graph_model.borrow().clone() };
        if let Err(e) = weight_graph_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload weight data: {}",
                e
            );
        }

        let view = self.upcast_ref::<View>();
        view.set_title(i18n_f("Current BMI: {}", &[&self.bmi(&weight_graph_model)]));
        self.update_weight_goal_label(&weight_graph_model);

        if let Some(view) = self_.weight_graph_view.get() {
            view.set_points(weight_graph_model.to_points());
        } else if !weight_graph_model.is_empty() {
            let weight_graph_view = GraphView::new();
            weight_graph_view.set_points(weight_graph_model.to_points());
            weight_graph_view.set_x_lines_interval(10.0);
            let settings = self_.settings.clone();
            weight_graph_view.set_hover_func(Some(Box::new(move |p| {
                if settings.unit_system() == UnitSystem::Imperial {
                    // TRANSLATORS: Weight X on date Y
                    ni18n_f("{} pound on {}",  "{} pounds on {}", p.value as u32, &[&p.value.to_string(), &p.date.format_local()])
                } else {
                    // TRANSLATORS: Weight X on date Y
                    ni18n_f("{} kilogram on {}", "{} kilograms on {}", p.value as u32, &[&p.value.to_string(), &p.date.format_local()])
                }
            })));
            let unit_goal = self_.settings.user_weight_goal();
            let weight_goal = if self_.settings.unit_system() == UnitSystem::Imperial {
                unit_goal.get::<pound>()
            } else {
                unit_goal.get::<kilogram>()
            };
            weight_graph_view.set_limit(Some(weight_goal));
            weight_graph_view.set_limit_label(Some(i18n("Weight goal")));

            self_.scrolled_window.set_child(Some(&weight_graph_view));
            view.stack().set_visible_child_name("data_page");

            self_.weight_graph_view.set(weight_graph_view).unwrap();

            self_.settings_handler_id.replace(Some(
                self_.settings.connect_user_weight_goal_changed(
                    glib::clone!(@weak self as obj => move |_,_| {
                        gtk_macros::spawn!(async move {
                            obj.update().await;
                        });
                    }),
                ),
            ));
        }

        self_.weight_graph_model.replace(weight_graph_model);
    }

    fn bmi(&self, model: &GraphModelWeight) -> String {
        if let Some(last_weight) = model.last_weight() {
            let height = self.imp().settings.user_height().get::<meter>() as f32;
            let bmi =
                (last_weight.get::<kilogram>() as f32 / (height * height)).round_decimal_places(1);
            format!("{bmi:.1}", bmi = bmi)
        } else {
            i18n("Unknown BMI")
        }
    }

    fn imp(&self) -> &imp::ViewWeight {
        imp::ViewWeight::from_instance(self)
    }

    // TRANSLATORS notes have to be on the same line, so we cant split them
    #[rustfmt::skip]
    fn update_weight_goal_label(&self, model: &GraphModelWeight) {
        let self_ = self.imp();
        let weight_goal = self_.settings.user_weight_goal();
        let unit_system = self_.settings.unit_system();
        let weight_value = if unit_system == UnitSystem::Imperial {
            weight_goal.get::<pound>()
        } else {
            weight_goal.get::<kilogram>()
        };
        let goal_label = self.upcast_ref::<View>().goal_label();

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
            goal_label.set_text(&goal_label_text);
        } else if weight_value > 0.1 && !model.is_empty() {
            if model.last_weight().unwrap() == weight_goal {
                goal_label.set_text(&i18n("You've reached your weight goal. Great job!"));
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
                // TRANSLATORS: First part of message, ends with [...] you have {} pound left to reach it[.] See next source string.
                ni18n_f("Your weight goal is {} pound,",
                    "Your weight goal is {} pounds,",
                    weight_value as u32, &[&format!("{weight_value:.1}",
                    weight_value = weight_value)],
                ) +
                // TRANSLATORS: Second (final) part of message, see previous source string.
                &ni18n_f( "you have {} pound left to reach it",
                    "you have {} pounds left to reach it",
                    diff as u32, &[&format!("{diff:.1}",
                    diff = diff.round_decimal_places(1))],
                )
            } else {
                // TRANSLATORS: First part of message, ends with [...] you have {} kilogram left to reach it[.] See next source string.
                ni18n_f("Your weight goal is {} kilogram,",
                    "Your weight goal is {} kilograms,",
                    weight_value as u32,
                    &[&format!("{weight_value:.1}", weight_value = weight_value)],
                ) +
                // TRANSLATORS: Second (final) part of message, see previous source string.
                &ni18n_f("you have {} kilogram left to reach it",
                    "you have {} kilograms left to reach it",
                    diff as u32,
                    &[&format!("{diff:.1}", diff = diff.round_decimal_places(1))],
                )
            };
            goal_label.set_text(&goal_label_text);
        } else {
            goal_label.set_text(&i18n(
                "No weight goal set yet. You can set it in Health's preferences.",
            ));
        }
    }
}
