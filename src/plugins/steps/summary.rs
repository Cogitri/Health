use std::convert::TryInto;

use crate::{
    core::{Database, Settings},
    ni18n_f,
    plugins::PluginSummaryRow,
};
use adw::prelude::*;
use gtk::{glib, subclass::prelude::*};

mod imp {
    use crate::{
        plugins::{summary::PinnedResultFuture, PluginSummaryRow, PluginSummaryRowImpl},
        widgets::CircularProgressBar,
    };
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/steps/summary.ui")]
    pub struct PluginStepsSummaryRow {
        #[template_child]
        pub circular_progress_bar: TemplateChild<CircularProgressBar>,
        #[template_child]
        pub steps_percentage: TemplateChild<gtk::Label>,
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

    impl ObjectImpl for PluginStepsSummaryRow {}
    impl WidgetImpl for PluginStepsSummaryRow {}
    impl ListBoxRowImpl for PluginStepsSummaryRow {}
    impl PreferencesRowImpl for PluginStepsSummaryRow {}
    impl ActionRowImpl for PluginStepsSummaryRow {}
    impl PluginSummaryRowImpl for PluginStepsSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginStepsSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance over their step count.
    pub struct PluginStepsSummaryRow(ObjectSubclass<imp::PluginStepsSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginStepsSummaryRow {
    pub fn new(name: &str) -> Self {
        glib::Object::new(&[("plugin-name", &name)])
            .expect("Failed to create PluginStepsSummaryRow")
    }

    fn imp(&self) -> &imp::PluginStepsSummaryRow {
        imp::PluginStepsSummaryRow::from_instance(self)
    }

    pub async fn update(&self) {
        let self_ = self.imp();
        let db = Database::instance();
        let step_count = db
            .todays_steps(chrono::Local::today().and_hms(0, 0, 0).into())
            .await
            .unwrap_or(0);
        let step_goal = Settings::instance().user_step_goal();

        self.set_subtitle(&ni18n_f(
            "{} step taken today",
            "{} steps taken today",
            step_count.try_into().unwrap(),
            &[&step_count.to_string()],
        ));
        self_.circular_progress_bar.set_step_goal(step_goal.into());
        self_.circular_progress_bar.set_step_count(step_count);
        self_.steps_percentage.set_text(&format!(
            "{}%",
            &((step_count / i64::from(step_goal.min(1)) * 100) as u32)
        ));
    }
}

#[cfg(test)]
mod test {
    use super::PluginStepsSummaryRow;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginStepsSummaryRow::new("");
    }
}
