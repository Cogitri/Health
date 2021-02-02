use crate::{core::Database, model::GraphModelWeight, views::View};
use gdk::subclass::prelude::*;

mod imp {
    use crate::{
        core::{i18n, i18n_f, settings::Unitsystem, Settings},
        model::GraphModelWeight,
        views::{GraphView, View},
    };
    use chrono::Duration;
    use glib::{subclass, Cast};
    use gtk::{subclass::prelude::*, CompositeTemplate, WidgetExt};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;
    use uom::si::{
        length::meter,
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/weight_view.ui")]
    pub struct ViewWeight {
        settings: Settings,
        weight_graph_view: OnceCell<GraphView>,
        weight_graph_model: OnceCell<RefCell<GraphModelWeight>>,
    }

    impl ObjectSubclass for ViewWeight {
        const NAME: &'static str = "HealthViewWeight";
        type ParentType = View;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ViewWeight;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: Settings::new(),
                weight_graph_view: OnceCell::new(),
                weight_graph_model: OnceCell::new(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
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
    }

    impl ViewWeight {
        pub fn set_weight_graph_model(&self, graph_model: GraphModelWeight) {
            self.weight_graph_model
                .set(RefCell::new(graph_model))
                .unwrap();
        }

        fn update_weightgoal_label(
            &self,
            obj: &crate::views::ViewWeight,
            model: &GraphModelWeight,
        ) {
            let weightgoal = self.settings.get_user_weightgoal();
            let unitsystem = self.settings.get_unitsystem();
            let (weight_value, translation) = if unitsystem == Unitsystem::Imperial {
                (weightgoal.get::<pound>(), i18n("pounds"))
            } else {
                (weightgoal.get::<kilogram>(), i18n("kilogram"))
            };
            let goal_label = obj.upcast_ref::<View>().get_goal_label();

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
                        &format!("{diff:.2}", diff = diff),
                        &translation,
                        &format!("{weight_value:.2}", weight_value = weight_value),
                        &translation,
                    ],
                ));
            } else {
                goal_label.set_text(&i18n(
                    "No weightgoal set yet. You can set it in Health's preferences.",
                ));
            }
        }

        fn get_bmi(&self, model: &GraphModelWeight) -> String {
            if let Some(last_weight) = model.get_last_weight() {
                let height = self.settings.get_user_height().get::<meter>() as f32;
                let bmi = last_weight.get::<kilogram>() as f32 / (height * height);
                format!("{bmi:.2}", bmi = bmi)
            } else {
                i18n("Unknown BMI")
            }
        }

        pub async fn update(&self, obj: &super::ViewWeight) {
            let mut weight_graph_model = self.weight_graph_model.get().unwrap().borrow_mut();
            if let Err(e) = weight_graph_model.reload(Duration::days(30)).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to reload weight data: {}",
                    e
                );
            }

            let view = obj.upcast_ref::<View>();
            view.set_title(i18n_f(
                "Current BMI: {}",
                &[&self.get_bmi(&weight_graph_model)],
            ));
            self.update_weightgoal_label(obj, &weight_graph_model);

            if let Some(view) = self.weight_graph_view.get() {
                view.set_points(weight_graph_model.to_points());
            } else if !weight_graph_model.is_empty() {
                let weight_graph_view = GraphView::new();
                weight_graph_view.set_points(weight_graph_model.to_points());
                weight_graph_view.set_x_lines_interval(10.0);
                let settings = self.settings.clone();
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
                let unitgoal = self.settings.get_user_weightgoal();
                let weightgoal = if self.settings.get_unitsystem() == Unitsystem::Imperial {
                    unitgoal.get::<pound>()
                } else {
                    unitgoal.get::<kilogram>()
                };
                weight_graph_view.set_limit(Some(weightgoal));
                weight_graph_view.set_limit_label(Some(i18n("Weightgoal")));

                view.get_scrolled_window()
                    .set_child(Some(&weight_graph_view));
                view.get_stack().set_visible_child_name("data_page");

                self.weight_graph_view.set(weight_graph_view).unwrap();

                self.settings.connect_user_weightgoal_changed(
                    glib::clone!(@weak obj => move |_,_| {
                        glib::MainContext::default().spawn_local(async move {
                            ViewWeight::from_instance(&obj).update(&obj).await
                        })
                    }),
                );
            }
        }
    }
}

glib::wrapper! {
    pub struct ViewWeight(ObjectSubclass<imp::ViewWeight>)
        @extends View;
}

impl ViewWeight {
    pub fn new(database: Database) -> Self {
        let o = glib::Object::new(&[]).expect("Failed to create ViewWeight");

        imp::ViewWeight::from_instance(&o)
            .set_weight_graph_model(GraphModelWeight::new(database.clone()));

        database.connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    pub async fn update(&self) {
        imp::ViewWeight::from_instance(self).update(self).await;
    }
}
