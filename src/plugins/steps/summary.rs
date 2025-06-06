use std::convert::TryInto;

use crate::{
    core::{i18n, ni18n_f, Database, Settings},
    plugins::{PluginName, PluginSummaryRow, PluginSummaryRowExt},
};
use adw::prelude::*;
use gtk::{glib, subclass::prelude::*};

mod imp {
    use crate::{
        core::Database, plugins::PluginSummaryRow, prelude::*, widgets::CircularProgressBar,
    };
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/steps/summary.ui")]
    pub struct PluginStepsSummaryRow {
        #[template_child]
        pub circular_progress_bar: TemplateChild<CircularProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginStepsSummaryRow {
        const NAME: &'static str = "HealthPluginStepsSummaryRow";
        type ParentType = PluginSummaryRow;
        type Type = super::PluginStepsSummaryRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginStepsSummaryRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            Database::instance().connect_activities_updated(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    gtk_macros::spawn!(async move {
                        obj.update().await;
                    });
                }
            ));
        }
    }
    impl WidgetImpl for PluginStepsSummaryRow {}
    impl ListBoxRowImpl for PluginStepsSummaryRow {}
    impl PreferencesRowImpl for PluginStepsSummaryRow {}
    impl ActionRowImpl for PluginStepsSummaryRow {}
    impl PluginSummaryRowImpl for PluginStepsSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _, send| {
                        gtk_macros::spawn!(async move {
                            obj.downcast_ref::<super::PluginStepsSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance over their step count.
    pub struct PluginStepsSummaryRow(ObjectSubclass<imp::PluginStepsSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginStepsSummaryRow {
    pub fn new(name: PluginName) -> Self {
        let obj: Self = glib::Object::builder()
            .property("plugin-name", &name)
            .property("activatable", true)
            .build();

        obj.bind_right_click();

        obj
    }

    pub async fn update(&self) {
        let imp = self.imp();
        let db = Database::instance();
        let user_id = i64::from(Settings::instance().active_user_id());
        let user = &db.user(user_id).await.unwrap();
        let step_count = db.todays_steps().await.unwrap_or(0);
        let step_goal = user.user_stepgoal().unwrap_or(0) as u32;

        // Translators: Steps taken today. Example: "5000 steps"
        self.set_subtitle(&ni18n_f(
            "{} step",
            "{} steps",
            step_count.try_into().unwrap(),
            &[&step_count.to_string()],
        ));
        imp.circular_progress_bar.set_step_goal(step_goal);
        imp.circular_progress_bar
            .set_step_count(step_count.try_into().unwrap());
        let steps_percentage = (step_count as f32 / step_goal.max(1) as f32 * 100.0) as u32;
        if steps_percentage < 100 {
            self.set_tooltip_text(Some(&ni18n_f(
                "Reached {} percent of daily step goal",
                "Reached {} percent of daily step goal",
                steps_percentage,
                &[&steps_percentage.to_string()],
            )));
        } else {
            self.set_tooltip_text(Some(&i18n(
                "Well done! You have reached your daily step goal!",
            )));
        }
    }
}

#[cfg(test)]
mod test {
    use super::{PluginName, PluginStepsSummaryRow};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginStepsSummaryRow::new(PluginName::Steps);
    }
}
