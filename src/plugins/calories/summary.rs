use crate::{
    core::i18n_f,
    core::Database,
    plugins::{PluginName, PluginSummaryRow},
    prelude::*,
};
use adw::prelude::*;
use gtk::glib;

mod imp {
    use crate::{core::Database, plugins::PluginSummaryRow, prelude::*};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, CompositeTemplate};

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

    impl ObjectImpl for PluginCaloriesSummaryRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            Database::instance().connect_weights_updated(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    gtk_macros::spawn!(async move {
                        obj.update().await;
                    });
                }
            ));
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
    impl WidgetImpl for PluginCaloriesSummaryRow {}
    impl ListBoxRowImpl for PluginCaloriesSummaryRow {}
    impl PreferencesRowImpl for PluginCaloriesSummaryRow {}
    impl ActionRowImpl for PluginCaloriesSummaryRow {}
    impl PluginSummaryRowImpl for PluginCaloriesSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _, send| {
                        gtk_macros::spawn!(async move {
                            obj.downcast_ref::<super::PluginCaloriesSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance over calorie burn rate.
    pub struct PluginCaloriesSummaryRow(ObjectSubclass<imp::PluginCaloriesSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginCaloriesSummaryRow {
    pub fn new(name: PluginName) -> Self {
        let obj: Self = glib::Object::builder()
            .property("plugin-name", &name)
            .property("activatable", true)
            .build();

        obj.bind_right_click();

        obj
    }

    pub async fn update(&self) {
        if let Some(bar) = Database::instance()
            .calories(glib::DateTime::local().add_days(-1).unwrap())
            .await
            .ok()
            .and_then(|s| s.first().cloned())
        {
            let calories_burned_today: i64 = bar.calorie_split.values().sum();
            // Translators: cal is short for calories burned today. Example: "2666 cal"
            self.set_subtitle(&i18n_f("{} cal", &[&calories_burned_today.to_string()]))
        } else {
            // Translators: cal is short for calories burned today. Example: "2666 cal"
            self.set_subtitle(&i18n_f("{} cal", &["0"]))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{PluginCaloriesSummaryRow, PluginName};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginCaloriesSummaryRow::new(PluginName::Calories);
    }
}
