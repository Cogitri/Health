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
    core::Database,
    plugins::{
        activities::{ModelActivity, ModelActivityMocked},
        PluginDetails, PluginDetailsExt,
    },
};
use gtk::glib::{self, subclass::prelude::*};
use gtk_macros::spawn;

use self::imp::DataProvider;

mod imp {
    use crate::{
        model::Activity,
        plugins::activities::{ModelActivity, ModelActivityMocked},
        plugins::{details::PinnedResultFuture, PluginDetails, PluginDetailsImpl},
        widgets::ActivityRow,
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

    #[derive(Debug, Clone)]
    pub enum DataProvider {
        Actual(ModelActivity),
        Mocked(ModelActivityMocked),
    }

    impl Default for DataProvider {
        fn default() -> Self {
            Self::Actual(ModelActivity::default())
        }
    }

    impl DataProvider {
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

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/activities/details.ui")]
    pub struct PluginActivitiesDetails {
        pub activity_model: RefCell<DataProvider>,
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
            if let DataProvider::Actual(m) = &*self.activity_model.borrow() {
                let selection_model = gtk::NoSelection::new(Some(m));
                let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));
                self.frame
                    .set_child(Some(list_view.upcast_ref::<gtk::Widget>()));
                list_view.style_context().add_class("content");
                self.activities_list_view.set(list_view).unwrap();
            } else {
                unimplemented!();
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
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct PluginActivitiesDetails(ObjectSubclass<imp::PluginActivitiesDetails>)
        @extends gtk::Widget, adw::Bin, PluginDetails,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginActivitiesDetails {
    pub fn mock(&self) {
        let m = ModelActivityMocked::new();
        self.imp()
            .activities_list_view
            .get()
            .unwrap()
            .set_model(Some(&gtk::NoSelection::new(Some(&m))));
        self.imp().activity_model.replace(DataProvider::Mocked(m));
        spawn!(glib::clone!(@weak self as obj => async move {
            obj.update().await;
        }));
    }

    pub fn unmock(&self) {
        let m = ModelActivity::new();
        self.imp()
            .activities_list_view
            .get()
            .unwrap()
            .set_model(Some(&gtk::NoSelection::new(Some(&m))));
        self.imp().activity_model.replace(DataProvider::Actual(m));
        spawn!(glib::clone!(@weak self as obj => async move {
            obj.update().await;
        }));
    }

    /// Create a new [PluginActivitiesDetails] to display previous activities.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create PluginActivitiesDetails");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    /// Reload the [ModelActivity](crate::model::ModelActivity)'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = { (*self.imp().activity_model.borrow()).clone() };
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
