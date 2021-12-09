use crate::{core::Database, ni18n_f, plugins::PluginSummaryRow};
use gtk::{glib, subclass::prelude::*};

mod imp {
    use crate::plugins::{PinnedResultFuture, PluginSummaryRow, PluginSummaryRowImpl};
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

    impl ObjectImpl for PluginActivitiesSummaryRow {}
    impl WidgetImpl for PluginActivitiesSummaryRow {}
    impl ListBoxRowImpl for PluginActivitiesSummaryRow {}
    impl PreferencesRowImpl for PluginActivitiesSummaryRow {}
    impl ActionRowImpl for PluginActivitiesSummaryRow {}
    impl PluginSummaryRowImpl for PluginActivitiesSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture {
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
    /// An implementation of [View] visualizes streak counts and daily step records.
    pub struct PluginActivitiesSummaryRow(ObjectSubclass<imp::PluginActivitiesSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginActivitiesSummaryRow {
    pub fn new(name: &str) -> Self {
        glib::Object::new(&[("plugin-name", &name)])
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
