use crate::{core::HealthDatabase, model::HealthGraphModelWeight, views::HealthView};
use gdk::subclass::prelude::*;

mod imp {
    use super::*;
    use crate::core::{i18n, i18n_f, settings::Unitsystem, HealthSettings};
    use crate::views::HealthGraphView;
    use chrono::Duration;
    use glib::{subclass, Cast};
    use gtk::{subclass::prelude::*, CompositeTemplate, WidgetExt};
    use std::cell::RefCell;
    use uom::si::{
        length::meter,
        mass::{kilogram, pound},
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/weight_view.ui")]
    pub struct HealthViewWeight {
        settings: HealthSettings,
        weight_graph_view: Option<HealthGraphView>,
        weight_graph_model: RefCell<Option<HealthGraphModelWeight>>,
    }

    impl ObjectSubclass for HealthViewWeight {
        const NAME: &'static str = "HealthViewWeight";
        type ParentType = HealthView;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthViewWeight;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: HealthSettings::new(),
                weight_graph_view: None,
                weight_graph_model: RefCell::new(None),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<HealthView>().init_template();
            }
        }
    }

    impl WidgetImpl for HealthViewWeight {}

    impl ObjectImpl for HealthViewWeight {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl HealthViewWeight {
        pub fn set_weight_graph_model(&self, graph_model: HealthGraphModelWeight) {
            self.weight_graph_model.replace(Some(graph_model));
        }

        fn update_weightgoal_label(
            &self,
            obj: &crate::views::HealthViewWeight,
            model: &HealthGraphModelWeight,
        ) {
            let weightgoal = self.settings.get_user_weightgoal();
            let unitsystem = self.settings.get_unitsystem();
            let (weight_value, translation) = if unitsystem == Unitsystem::Imperial {
                (weightgoal.get::<pound>(), i18n("pounds"))
            } else {
                (weightgoal.get::<kilogram>(), i18n("kilogram"))
            };
            let goal_label = obj.upcast_ref::<HealthView>().get_goal_label();

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

        fn get_bmi(&self, model: &HealthGraphModelWeight) -> String {
            if let Some(last_weight) = model.get_last_weight() {
                let height = self.settings.get_user_height().get::<meter>() as f32;
                let bmi = last_weight.get::<kilogram>() as f32 / (height * height);
                format!("{bmi:.2}", bmi = bmi)
            } else {
                i18n("Unknown BMI")
            }
        }

        pub async fn update(&self, obj: &super::HealthViewWeight) {
            let mut weight_graph_model_ref = self.weight_graph_model.borrow_mut();
            let weight_graph_model = weight_graph_model_ref.as_mut().unwrap();
            if let Err(e) = weight_graph_model.reload(Duration::days(30)).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to reload weight data: {}",
                    e
                );
            }

            let view = obj.upcast_ref::<HealthView>();
            view.set_title(i18n_f(
                "Current BMI: {}",
                &[&self.get_bmi(&weight_graph_model)],
            ));
            self.update_weightgoal_label(obj, &weight_graph_model);

            if let Some(view) = &self.weight_graph_view {
                view.set_points(weight_graph_model.to_points());
            } else if !weight_graph_model.is_empty() {
                let weight_graph_view = HealthGraphView::new();
                weight_graph_view.set_points(weight_graph_model.to_points());
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

                self.settings.connect_user_weightgoal_changed(
                    glib::clone!(@weak obj => move |_,_| {
                        glib::MainContext::default().spawn_local(async move {
                            HealthViewWeight::from_instance(&obj).update(&obj).await
                        })
                    }),
                );
            }
        }
    }
}

glib::wrapper! {
    pub struct HealthViewWeight(ObjectSubclass<imp::HealthViewWeight>)
        @extends HealthView;
}

impl HealthViewWeight {
    pub fn new(database: HealthDatabase) -> Self {
        let o = glib::Object::new(&[]).expect("Failed to create HealthViewWeight");

        imp::HealthViewWeight::from_instance(&o)
            .set_weight_graph_model(HealthGraphModelWeight::new(database));

        o
    }

    pub async fn update(&self) {
        imp::HealthViewWeight::from_instance(self)
            .update(self)
            .await;
    }
}
