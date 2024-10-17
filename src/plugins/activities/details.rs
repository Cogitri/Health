/* view_activity.rs
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

use crate::{
    plugins::{
        activities::{ModelActivity, ModelActivityMocked},
        PluginDetails,
    },
    prelude::*,
};
use gtk::glib::{self, subclass::prelude::*, Boxed};

mod imp {
    use super::{DataProvider, DataProviderBoxed};
    use crate::{
        core::Database, model::Activity, plugins::PluginDetails, prelude::*, widgets::ActivityRow,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        self, gio,
        glib::{self, Cast},
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/activities/details.ui")]
    pub struct PluginActivitiesDetails {
        pub activity_model: RefCell<Option<DataProvider>>,
        #[template_child]
        pub list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub stack_activity: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginActivitiesDetails {
        const NAME: &'static str = "HealthPluginActivitiesDetails";
        type ParentType = PluginDetails;
        type Type = super::PluginActivitiesDetails;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginActivitiesDetails {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            let m: gio::ListModel = match &*self.activity_model.borrow() {
                Some(DataProvider::Actual(m)) => m.clone().upcast(),
                Some(DataProvider::Mocked(m)) => m.clone().upcast(),
                None => unimplemented!(),
            };
            self.list_box.bind_model(Some(&m), |obj| {
                ActivityRow::new(obj.downcast_ref::<Activity>().unwrap()).upcast()
            });

            Database::instance().connect_activities_updated(glib::clone!(@weak obj => move |_| {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<DataProviderBoxed>("data-provider")
                        .construct()
                        .write_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "data-provider" => {
                    self.activity_model
                        .replace(Some(value.get::<DataProviderBoxed>().unwrap().0));
                }
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for PluginActivitiesDetails {}
    impl BinImpl for PluginActivitiesDetails {}

    impl PluginDetailsImpl for PluginActivitiesDetails {
        fn update(&self, obj: &PluginDetails) -> PinnedResultFuture<()> {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::PluginActivitiesDetails>()
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
    /// An implementation of [PluginDetails] that visualizes activities the user recently did.
    pub struct PluginActivitiesDetails(ObjectSubclass<imp::PluginActivitiesDetails>)
        @extends gtk::Widget, adw::Bin, PluginDetails,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginActivitiesDetails {
    /// Create a new [PluginActivitiesDetails] to display previous activities.
    pub fn new(data_provider: DataProvider) -> Self {
        glib::Object::builder()
            .property(
                "is-mocked",
                matches!(data_provider, DataProvider::Mocked(_)),
            )
            .property("data-provider", DataProviderBoxed(data_provider))
            .build()
    }

    /// Reload the [ModelActivity](crate::plugins::activities::ModelActivity)'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = { (*self.imp().activity_model.borrow()).clone().unwrap() };
        let reload_result = activity_model.reload().await;
        if let Err(e) = reload_result {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload activity data: {e}",
            );
        };

        if activity_model.activity_present().await {
            self.switch_to_data_page();
            self.imp()
                .stack_activity
                .set_visible_child_name(if !activity_model.is_empty() {
                    "recent_activities"
                } else {
                    "no_recent"
                });
        } else {
            self.switch_to_empty_page();
        }
    }
}

#[derive(Clone, Boxed)]
#[boxed_type(name = "HealthDataProviderActivitiesBoxed")]
pub struct DataProviderBoxed(DataProvider);

#[derive(Debug, Clone)]
pub enum DataProvider {
    Actual(ModelActivity),
    Mocked(ModelActivityMocked),
}

impl DataProvider {
    pub fn actual() -> Self {
        Self::Actual(ModelActivity::new())
    }

    pub fn mocked() -> Self {
        Self::Mocked(ModelActivityMocked::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Actual(m) => m.is_empty(),
            Self::Mocked(m) => m.is_empty(),
        }
    }

    pub async fn reload(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(m) => m.reload().await,
            Self::Mocked(m) => m.reload().await,
        }
    }

    pub async fn activity_present(&self) -> bool {
        match self {
            Self::Actual(m) => m.activity_present().await,
            Self::Mocked(m) => m.activity_present().await,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DataProvider, PluginActivitiesDetails};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginActivitiesDetails::new(DataProvider::mocked());
    }
}
