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

use crate::plugins::{
    activities::{ModelActivity, ModelActivityMocked},
    PluginDetails, PluginDetailsExt,
};
use gtk::glib::{self, subclass::prelude::*, Boxed};

mod imp {
    use super::{DataProvider, DataProviderBoxed};
    use crate::{
        model::Activity,
        plugins::{details::PinnedResultFuture, PluginDetails, PluginDetailsImpl},
        widgets::ActivityRow,
        Database,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        self, gio,
        glib::{self, Cast},
        subclass::prelude::*,
        CompositeTemplate,
    };
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/activities/details.ui")]
    pub struct PluginActivitiesDetails {
        pub activity_model: RefCell<Option<DataProvider>>,
        pub activities_list_view: OnceCell<gtk::ListView>,
        #[template_child]
        pub frame: TemplateChild<gtk::Frame>,
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
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<PluginDetails>().init_template();
            }
        }
    }

    impl ObjectImpl for PluginActivitiesDetails {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(move |_, item| item.set_child(Some(&ActivityRow::new())));
            factory.connect_bind(move |_, list_item| {
                let activity = list_item.item().unwrap().downcast::<Activity>().unwrap();

                let child = list_item
                    .child()
                    .unwrap()
                    .downcast::<ActivityRow>()
                    .unwrap();
                child.set_activity(activity);
            });
            let m: gio::ListModel = match &*self.activity_model.borrow() {
                Some(DataProvider::Actual(m)) => m.clone().upcast(),
                Some(DataProvider::Mocked(m)) => m.clone().upcast(),
                None => unimplemented!(),
            };
            let selection_model = gtk::NoSelection::new(Some(&m));
            let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));
            self.frame
                .set_child(Some(list_view.upcast_ref::<gtk::Widget>()));
            list_view.style_context().add_class("content");
            self.activities_list_view.set(list_view).unwrap();

            Database::instance().connect_activities_updated(glib::clone!(@weak obj => move || {
                gtk_macros::spawn!(async move {
                    obj.update().await;
                });
            }));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "data-provider",
                    "data-provider",
                    "data-provider",
                    DataProviderBoxed::static_type(),
                    glib::ParamFlags::CONSTRUCT | glib::ParamFlags::WRITABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
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
        fn update_actual(&self, obj: &PluginDetails) -> PinnedResultFuture {
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
        glib::Object::new(&[("data-provider", &DataProviderBoxed(data_provider))])
            .expect("Failed to create PluginActivitiesDetails")
    }

    /// Reload the [ModelActivity](crate::plugins::activities::ModelActivity)'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = { (*self.imp().activity_model.borrow()).clone().unwrap() };
        let reload_result = activity_model.reload().await;
        if let Err(e) = reload_result {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload activity data: {}",
                e
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

    fn imp(&self) -> &imp::PluginActivitiesDetails {
        imp::PluginActivitiesDetails::from_instance(self)
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

    #[test]
    fn new() {
        init_gtk();
        PluginActivitiesDetails::new(DataProvider::mocked());
    }
}
