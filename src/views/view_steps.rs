/* view_steps.rs
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

use crate::{core::Database, model::GraphModelSteps, views::View};
use glib::subclass::types::ObjectSubclass;

mod imp {
    use crate::{
        core::{i18n, i18n_f, Settings},
        model::GraphModelSteps,
        views::{GraphView, View},
    };
    use chrono::Duration;
    use glib::{subclass, Cast};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/step_view.ui")]
    pub struct ViewSteps {
        #[template_child]
        scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        settings: Settings,
        steps_graph_view: OnceCell<GraphView>,
        steps_graph_model: OnceCell<RefCell<GraphModelSteps>>,
    }

    impl ObjectSubclass for ViewSteps {
        const NAME: &'static str = "HealthViewSteps";
        type ParentType = View;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ViewSteps;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            let settings = Settings::new();

            Self {
                scrolled_window: TemplateChild::default(),
                settings,
                steps_graph_view: OnceCell::new(),
                steps_graph_model: OnceCell::new(),
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

    impl WidgetImpl for ViewSteps {}

    impl ObjectImpl for ViewSteps {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }
    }

    impl ViewSteps {
        pub fn set_steps_graph_model(&self, model: GraphModelSteps) {
            self.steps_graph_model.set(RefCell::new(model)).unwrap();
        }

        pub async fn update(&self, obj: &super::ViewSteps) {
            let mut steps_graph_model = { self.steps_graph_model.get().unwrap().borrow().clone() };
            if let Err(e) = steps_graph_model.reload(Duration::days(30)).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to reload step data: {}",
                    e
                );
            }

            let view = obj.upcast_ref::<View>();
            view.set_title(i18n_f(
                "Today's steps: {}",
                &[&steps_graph_model
                    .get_today_step_count()
                    .unwrap_or(0)
                    .to_string()],
            ));

            let goal_label = view.get_goal_label();
            let streak_count =
                steps_graph_model.get_streak_count_today(self.settings.get_user_stepgoal());

            match streak_count {
                0 => {
                    let previous_streak = steps_graph_model
                        .get_streak_count_yesterday(self.settings.get_user_stepgoal());
                    if previous_streak == 0 {
                        goal_label.set_text (&i18n("No streak yet. Reach your stepgoal for multiple days to start a streak!"));
                    } else {
                        goal_label.set_text (&i18n_f("You're on a streak for {} days. Reach your stepgoal today to continue it!", &[&previous_streak.to_string()]));
                    }
                }
                1 => goal_label.set_text(&i18n(
                    "You've reached your stepgoal today. Keep going to start a streak!",
                )),
                _ => goal_label.set_text(&i18n_f(
                    "You're on a streak for {} days. Good job!",
                    &[&streak_count.to_string()],
                )),
            }

            if let Some(view) = self.steps_graph_view.get() {
                view.set_points(steps_graph_model.to_points());
            } else if !steps_graph_model.is_empty() {
                let steps_graph_view = GraphView::new();
                steps_graph_view.set_points(steps_graph_model.to_points());
                steps_graph_view.set_x_lines_interval(500.0);
                steps_graph_view.set_hover_func(Some(Box::new(|p| {
                    return i18n_f(
                        "{} steps on {}",
                        &[&p.value.to_string(), &format!("{}", p.date.format("%x"))],
                    );
                })));
                steps_graph_view.set_limit(Some(self.settings.get_user_stepgoal() as f32));
                steps_graph_view.set_limit_label(Some(i18n("Stepgoal")));

                self.scrolled_window.set_child(Some(&steps_graph_view));
                view.get_stack().set_visible_child_name("data_page");

                self.steps_graph_view.set(steps_graph_view).unwrap();

                self.settings
                    .connect_user_stepgoal_changed(glib::clone!(@weak obj => move |_,_| {
                        glib::MainContext::default().spawn_local(async move {
                            ViewSteps::from_instance(&obj).update(&obj).await
                        })
                    }));
            }

            self.steps_graph_model
                .get()
                .unwrap()
                .replace(steps_graph_model);
        }
    }
}

glib::wrapper! {
    pub struct ViewSteps(ObjectSubclass<imp::ViewSteps>)
        @extends View;
}

impl ViewSteps {
    pub fn new(database: Database) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewSteps");

        database.connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        imp::ViewSteps::from_instance(&o).set_steps_graph_model(GraphModelSteps::new(database));

        o
    }

    pub async fn update(&self) {
        imp::ViewSteps::from_instance(self).update(self).await;
    }
}
