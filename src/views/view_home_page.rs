/* view_home_page.rs
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
    core::{i18n, i18n_f, utils::prelude::*, Database, Unitsystem},
    model::{GraphModelSteps, GraphModelWeight},
    views::View,
    windows::ViewMode,
};
use adw::prelude::*;
use chrono::Duration;
use gtk::{
    glib::{self, clone, object::ObjectExt, subclass::prelude::*, Cast},
    prelude::*,
};
use uom::si::mass::{kilogram, pound};

mod imp {
    use crate::{
        views::View,
        widgets::{Arrows, CircularProgressBar, TabButton},
        windows::ViewMode,
        Settings,
    };
    use gtk::{
        glib::{self, subclass::Signal, Cast},
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct ViewHomePageMut {
        pub view_mode: ViewMode,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/home_page.ui")]
    pub struct ViewHomePage {
        pub inner: RefCell<ViewHomePageMut>,
        pub settings: Settings,
        pub settings_handler_id: RefCell<Option<glib::SignalHandlerId>>,
        pub settings_handler_id2: RefCell<Option<glib::SignalHandlerId>>,

        #[template_child]
        pub circular_progress_bar: TemplateChild<CircularProgressBar>,
        #[template_child]
        pub arrow: TemplateChild<Arrows>,
        #[template_child]
        pub button_flow_box: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub steps_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub weight_actionrow: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub steps_percentage: TemplateChild<gtk::Label>,
        #[template_child]
        pub weight_change: TemplateChild<gtk::Label>,
        #[template_child]
        pub weight_subtext: TemplateChild<gtk::Label>,
        #[template_child]
        pub arrow_box: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub tab_button: TemplateChild<TabButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewHomePage {
        const NAME: &'static str = "HealthViewHomePage";
        type ParentType = View;
        type Type = super::ViewHomePage;

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

    impl WidgetImpl for ViewHomePage {}

    impl ObjectImpl for ViewHomePage {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("view-changed", &[], glib::Type::UNIT.into()).build()]
            });

            SIGNALS.as_ref()
        }

        fn dispose(&self, _obj: &Self::Type) {
            if let Some(id) = self.settings_handler_id.borrow_mut().take() {
                self.settings.disconnect(id);
            }
            if let Some(id) = self.settings_handler_id2.borrow_mut().take() {
                self.settings.disconnect(id);
            }
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct ViewHomePage(ObjectSubclass<imp::ViewHomePage>)
        @extends View, gtk::Widget;
}

impl ViewHomePage {
    /// Connect to the `view-changed` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `view-changed` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_view_changed<F: Fn(ViewMode) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        let downgraded = self.downgrade();
        self.connect_local("view-changed", false, move |_| {
            let view_mode = downgraded
                .upgrade()
                .map_or(ViewMode::HomePage, |obj| obj.imp().inner.borrow().view_mode);
            callback(view_mode);
            None
        })
        .unwrap()
    }

    /// Create a new [ViewHomePage] to display previous activities.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewHomePage");
        o.upcast_ref::<View>()
            .stack()
            .set_visible_child_name("add_data_page");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update_activities().await;
            });
        }));

        Database::instance().connect_weights_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update_weights().await;
            });
        }));

        let self_ = o.imp();

        self_.button_flow_box.connect_child_activated(
            clone!(@weak o as obj => move |_,button_pressed| {
                obj.handle_button_press(button_pressed)
            }),
        );

        self_
            .settings_handler_id
            .replace(Some(self_.settings.connect_user_stepgoal_changed(
                glib::clone!(@weak o as obj => move |_,_| {
                    glib::MainContext::default().spawn_local(async move {
                        obj.update_activities().await
                    })
                }),
            )));

        self_
            .settings_handler_id2
            .replace(Some(self_.settings.connect_user_weightgoal_changed(
                glib::clone!(@weak o as obj => move |_,_| {
                    glib::MainContext::default().spawn_local(async move {
                        obj.update_weights().await
                    })
                }),
            )));

        o
    }

    fn handle_button_press(&self, button_pressed: &gtk::FlowBoxChild) {
        let self_ = self.imp();
        self_.inner.borrow_mut().view_mode =
            if button_pressed == &self_.button_flow_box.child_at_index(0).unwrap() {
                ViewMode::Weight
            } else if button_pressed == &self_.button_flow_box.child_at_index(1).unwrap() {
                ViewMode::Steps
            } else {
                ViewMode::Activities
            };
        self.emit_by_name("view-changed", &[]).unwrap();
    }

    pub async fn update_activities(&self) {
        let self_ = self.imp();
        self_
            .circular_progress_bar
            .set_step_goal(i64::from(self_.settings.user_stepgoal()));
        let mut steps_model = GraphModelSteps::new();
        if let Err(e) = steps_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload step data: {}",
                e
            );
        }
        let step_count = steps_model.today_step_count().unwrap_or(0);
        self_
            .circular_progress_bar
            .set_step_count(i64::from(step_count));
        self_.steps_actionrow.set_subtitle(Some(&i18n_f(
            "{} steps taken today",
            &[&step_count.to_string()],
        )));
        self_.steps_percentage.set_label(&i18n_f(
            "{}%",
            &[&(100 * step_count / self_.settings.user_stepgoal() as u32).to_string()],
        ));
    }

    pub async fn update_weights(&self) {
        let self_ = self.imp();
        let mut weight_model = GraphModelWeight::new();
        if let Err(e) = weight_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload step data: {}",
                e
            );
        }
        self_.arrow_box.set_visible(true);
        if !weight_model.is_empty() {
            let last_weight = if self_.settings.unitsystem() == Unitsystem::Imperial {
                weight_model.last_weight().unwrap().get::<pound>()
            } else {
                weight_model.last_weight().unwrap().get::<kilogram>()
            };
            let prev_weight = if self_.settings.unitsystem() == Unitsystem::Imperial {
                weight_model.penultimate_weight().unwrap().get::<pound>()
            } else {
                weight_model.penultimate_weight().unwrap().get::<kilogram>()
            };
            let last_weight_round = last_weight.round_decimal_places(1);
            self_.arrow.set_weight(last_weight_round);
            let difference = (last_weight - prev_weight).round_decimal_places(1);
            self_.arrow.set_weight_difference(difference);
            let subtitle = if self_.settings.unitsystem() == Unitsystem::Imperial {
                // TRANSLATORS: Current user weight
                i18n_f("{} pounds", &[&last_weight_round.to_string()])
            } else {
                // TRANSLATORS: Current user weight
                i18n_f("{} kilogram", &[&last_weight_round.to_string()])
            };
            self_.weight_actionrow.set_subtitle(Some(&subtitle));
            if difference > 0.0 {
                let label = if self_.settings.unitsystem() == Unitsystem::Imperial {
                    // TRANSLATORS: Difference to last weight measurement
                    i18n_f("+ {} pounds", &[&difference.to_string()])
                } else {
                    // TRANSLATORS: Difference to last weight measurement
                    i18n_f("+ {} kilogram", &[&difference.to_string()])
                };
                self_.weight_change.set_label(&label)
            } else if difference < 0.0 {
                let label = if self_.settings.unitsystem() == Unitsystem::Imperial {
                    // TRANSLATORS: Difference to last weight measurement
                    i18n_f("{} pounds", &[&difference.to_string()])
                } else {
                    // TRANSLATORS: Difference to last weight measurement
                    i18n_f("{} kilogram", &[&difference.to_string()])
                };
                self_.weight_change.set_label(&label)
            } else {
                self_.weight_change.set_label(&i18n("No change in weight"));
                self_.arrow_box.set_visible(false);
            }
            self_
                .weight_subtext
                .set_label(&i18n("compared to previous weight"));
        } else {
            self_
                .weight_subtext
                .set_label(&i18n("use + to add a weight record"));
            self_.arrow_box.set_visible(false);
        }
    }

    fn imp(&self) -> &imp::ViewHomePage {
        imp::ViewHomePage::from_instance(self)
    }
}
