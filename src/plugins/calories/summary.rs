use crate::{
    core::ni18n_f,
    core::Database,
    plugins::{PluginName, PluginSummaryRow},
};
use gtk::{glib, subclass::prelude::*};

mod imp {
    use crate::{plugins::PluginSummaryRow, prelude::*};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/calories/summary.ui")]
    pub struct PluginCaloriesSummaryRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginCaloriesSummaryRow {
        const NAME: &'static str = "HealthPluginCaloriesSummaryRow";
        type ParentType = PluginSummaryRow;
        type Type = super::PluginCaloriesSummaryRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginCaloriesSummaryRow {}
    impl WidgetImpl for PluginCaloriesSummaryRow {}
    impl ListBoxRowImpl for PluginCaloriesSummaryRow {}
    impl PreferencesRowImpl for PluginCaloriesSummaryRow {}
    impl ActionRowImpl for PluginCaloriesSummaryRow {}
    impl PluginSummaryRowImpl for PluginCaloriesSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginCaloriesSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance over calorie burn rate.
    pub struct PluginCaloriesSummaryRow(ObjectSubclass<imp::PluginCaloriesSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginCaloriesSummaryRow {
    pub fn new(name: PluginName) -> Self {
        glib::Object::new(&[("plugin-name", &name)])
            .expect("Failed to create PluginCaloriesSummaryRow")
    }

    fn imp(&self) -> &imp::PluginCaloriesSummaryRow {
        imp::PluginCaloriesSummaryRow::from_instance(self)
    }

    pub async fn update(&self) {
        let self_ = self.imp();
        if let Some(bar) = Database::instance()
            .calories((chrono::Local::now() - chrono::Duration::days(1)).into())
            .await
            .ok()
            .and_then(|s| s.get(0).cloned())
        {
            let calories_burned_today: i64 = bar.calorie_split.values().sum();
            self_.label.set_label(&ni18n_f(
                "{} calorie burned today",
                "{} calories burned today",
                calories_burned_today as u32,
                &[&calories_burned_today.to_string()],
            ))
        } else {
            self_.label.set_label(&ni18n_f(
                "{} calorie burned today",
                "{} calories burned today",
                0,
                &["0"],
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::PluginCaloriesSummaryRow;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginCaloriesSummaryRow::new("");
    }
}
