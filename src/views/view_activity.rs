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

use crate::{core::Database, views::View};
use gtk::glib::{self, subclass::prelude::*, Cast};
mod imp {
    use crate::{
        core::Settings,
        model::{Activity, ModelActivity},
        views::{PinnedResultFuture, View, ViewImpl},
        widgets::ActivityRow,
    };
    use gtk::{
        self, gio,
        glib::{self, Cast},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_view.ui")]
    pub struct ViewActivity {
        settings: Settings,
        pub activity_model: ModelActivity,
        pub activities_list_view: OnceCell<gtk::ListView>,
        #[template_child]
        pub frame: TemplateChild<gtk::Frame>,
        #[template_child]
        pub stack_activity: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewActivity {
        const NAME: &'static str = "HealthViewActivity";
        type ParentType = View;
        type Type = super::ViewActivity;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<View>().init_template();
            }
        }
    }

    impl WidgetImpl for ViewActivity {}

    impl ObjectImpl for ViewActivity {
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
            let selection_model = gtk::NoSelection::new(Some(&self.activity_model));
            let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));
            self.frame
                .set_child(Some(list_view.upcast_ref::<gtk::Widget>()));
            list_view.style_context().add_class("content");
            self.activities_list_view.set(list_view).unwrap();
        }
    }

    impl ViewImpl for ViewActivity {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<super::ViewActivity>()
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
    pub struct ViewActivity(ObjectSubclass<imp::ViewActivity>)
        @extends gtk::Widget, View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewActivity {
    /// Create a new [ViewActivity] to display previous activities.
    pub fn new() -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewActivity");

        Database::instance().connect_activities_updated(glib::clone!(@weak o => move || {
            gtk_macros::spawn!(async move {
                o.update().await;
            });
        }));

        o
    }

    /// Reload the [ModelActivity](crate::model::ModelActivity)'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = &self.imp().activity_model;
        let reload_result = activity_model.reload().await;
        if let Err(e) = reload_result {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload activity data: {}",
                e
            );
        };

        if activity_model.activity_present().await {
            self.upcast_ref::<View>()
                .stack()
                .set_visible_child_name("data_page");
            self.imp()
                .stack_activity
                .set_visible_child_name(if !activity_model.is_empty() {
                    "recent_activities"
                } else {
                    "no_recent"
                });
        }
    }

    fn imp(&self) -> &imp::ViewActivity {
        imp::ViewActivity::from_instance(self)
    }
}
