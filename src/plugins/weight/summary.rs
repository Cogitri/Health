use crate::{
    core::{i18n, ni18n_f, Settings, UnitSystem},
    model::WeightChange,
    plugins::weight::GraphModelWeight,
    plugins::{PluginName, PluginSummaryRow},
    prelude::*,
};
use adw::prelude::*;
use chrono::Duration;
use gtk::{glib, subclass::prelude::*};
use uom::si::mass::{kilogram, pound};

mod imp {
    use crate::{core::Database, plugins::PluginSummaryRow, prelude::*, widgets::Arrows};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/weight/summary.ui")]
    pub struct PluginWeightSummaryRow {
        #[template_child]
        pub weight_change: TemplateChild<gtk::Label>,
        #[template_child]
        pub weight_subtext: TemplateChild<gtk::Label>,
        #[template_child]
        pub arrow_box: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub arrow: TemplateChild<Arrows>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginWeightSummaryRow {
        const NAME: &'static str = "HealthPluginWeightSummaryRow";
        type ParentType = PluginSummaryRow;
        type Type = super::PluginWeightSummaryRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginWeightSummaryRow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            Database::instance().connect_weights_updated(glib::clone!(@weak obj => move || {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }
    }
    impl WidgetImpl for PluginWeightSummaryRow {}
    impl ListBoxRowImpl for PluginWeightSummaryRow {}
    impl PreferencesRowImpl for PluginWeightSummaryRow {}
    impl ActionRowImpl for PluginWeightSummaryRow {}
    impl PluginSummaryRowImpl for PluginWeightSummaryRow {
        fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginWeightSummaryRow>()
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
    /// An implementation of [PluginSummaryRow] giving the user a quick glance over their weight history.
    pub struct PluginWeightSummaryRow(ObjectSubclass<imp::PluginWeightSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow, PluginSummaryRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginWeightSummaryRow {
    pub fn new(name: PluginName) -> Self {
        glib::Object::new(&[("plugin-name", &name)])
            .expect("Failed to create PluginWeightSummaryRow")
    }

    fn imp(&self) -> &imp::PluginWeightSummaryRow {
        imp::PluginWeightSummaryRow::from_instance(self)
    }

    pub async fn update(&self) {
        let self_ = self.imp();
        let settings = Settings::instance();
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
            let last_weight = if settings.unit_system() == UnitSystem::Imperial {
                weight_model.last_weight().unwrap().get::<pound>()
            } else {
                weight_model.last_weight().unwrap().get::<kilogram>()
            };
            let prev_weight = if settings.unit_system() == UnitSystem::Imperial {
                weight_model.penultimate_weight().unwrap().get::<pound>()
            } else {
                weight_model.penultimate_weight().unwrap().get::<kilogram>()
            };
            let last_weight_round = last_weight.round_decimal_places(1);
            let difference = (last_weight - prev_weight).round_decimal_places(1);
            let change = if difference == 0.0 {
                WeightChange::NoChange
            } else if difference > 0.0 {
                WeightChange::Up
            } else {
                WeightChange::Down
            };
            self_.arrow.set_weight_change(change);
            let subtitle = if settings.unit_system() == UnitSystem::Imperial {
                // TRANSLATORS: Current user weight
                ni18n_f(
                    "{} pound",
                    "{} pounds",
                    last_weight_round as u32,
                    &[&last_weight_round.to_string()],
                )
            } else {
                // TRANSLATORS: Current user weight
                ni18n_f(
                    "{} kilogram",
                    "{} kilograms",
                    last_weight_round as u32,
                    &[&last_weight_round.to_string()],
                )
            };
            self.set_subtitle(&subtitle);
            if difference > 0.0 {
                let label = if settings.unit_system() == UnitSystem::Imperial {
                    // TRANSLATORS: Difference to last weight measurement
                    ni18n_f(
                        "+ {} pound",
                        "+ {} pounds",
                        difference as u32,
                        &[&difference.to_string()],
                    )
                } else {
                    // TRANSLATORS: Difference to last weight measurement
                    ni18n_f(
                        "+ {} kilogram",
                        "+ {} kilograms",
                        difference as u32,
                        &[&difference.to_string()],
                    )
                };
                self_.weight_change.set_label(&label)
            } else if difference < 0.0 {
                let label = if settings.unit_system() == UnitSystem::Imperial {
                    // TRANSLATORS: Difference to last weight measurement
                    ni18n_f(
                        "{} pound",
                        "{} pounds",
                        difference as u32,
                        &[&difference.to_string()],
                    )
                } else {
                    // TRANSLATORS: Difference to last weight measurement
                    ni18n_f(
                        "{} kilogram",
                        "{} kilograms",
                        difference as u32,
                        &[&difference.to_string()],
                    )
                };
                self_.weight_change.set_label(&label)
            } else {
                self_.weight_change.set_label(&i18n("No change in weight"));
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
}

#[cfg(test)]
mod test {
    use super::{PluginName, PluginWeightSummaryRow};
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginWeightSummaryRow::new(PluginName::Weight);
    }
}
