use crate::{
    core::ni18n_f,
    core::Database,
    plugins::{PluginName, PluginSummaryRow},
};
use gtk::{glib, subclass::prelude::*};

mod imp {
    use crate::{core::Database, plugins::PluginSummaryRow, prelude::*};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/activities/summary.ui")]
    pub struct PluginActivitiesSummaryRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginActivitiesSummaryRow {
        const NAME: &'static str = "HealthPluginActivitiesSummaryRow";
        type ParentType = PluginSummaryRow;
        type Type = super::PluginActivitiesSummaryRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginActivitiesSummaryRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            Database::instance().connect_activities_updated(glib::clone!(@weak obj => move |_| {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }
    }
    impl WidgetImpl for PluginActivitiesSummaryRow {}
    impl ListBoxRowImpl for PluginActivitiesSummaryRow {}
    impl PreferencesRowImpl for PluginActivitiesSummaryRow {}
    impl ActionRowImpl for PluginActivitiesSummaryRow {}
    impl PluginSummaryRowImpl for PluginActivitiesSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginActivitiesSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance at their active minutes.
    pub struct PluginActivitiesSummaryRow(ObjectSubclass<imp::PluginActivitiesSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginActivitiesSummaryRow {
    pub fn new(name: PluginName) -> Self {
        glib::Object::new(&[("plugin-name", &name.as_ref())])
            .expect("Failed to create PluginActivitiesSummaryRow")
    }

    fn imp(&self) -> &imp::PluginActivitiesSummaryRow {
        imp::PluginActivitiesSummaryRow::from_instance(self)
    }

    pub async fn update(&self) {
        let self_ = self.imp();
        let active_minutes_today: i64 = Database::instance()
            .activities(Some(
                (chrono::Local::now() - chrono::Duration::days(1)).into(),
            ))
            .await
            .unwrap_or_default()
            .iter()
            .map(|s| s.duration().num_minutes())
            .sum();
        self_.label.set_label(&ni18n_f(
            "{} active minutes today",
            "{} active minutes today",
            active_minutes_today as u32,
            &[&active_minutes_today.to_string()],
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{PluginActivitiesSummaryRow, PluginName};
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginActivitiesSummaryRow::new(PluginName::Activities);
    }
}
