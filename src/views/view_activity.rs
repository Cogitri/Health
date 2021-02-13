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
    model::{Activity, ModelActivity},
    views::View,
    widgets::ActivityRow,
};
use chrono::Duration;
use glib::{subclass::types::ObjectSubclass, Cast};

mod imp {
    use crate::{core::Settings, model::ModelActivity, views::View};
    use glib::{subclass, Cast};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_view.ui")]
    pub struct ViewActivity {
        settings: Settings,
        pub activity_model: OnceCell<ModelActivity>,
        #[template_child]
        pub activities_list_box: TemplateChild<gtk::ListBox>,
    }

    impl ObjectSubclass for ViewActivity {
        const NAME: &'static str = "HealthViewActivity";
        type ParentType = View;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ViewActivity;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                settings: Settings::new(),
                activity_model: OnceCell::new(),
                activities_list_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<View>().init_template();
            }
        }
    }

    impl WidgetImpl for ViewActivity {}

    impl ObjectImpl for ViewActivity {}
}

glib::wrapper! {
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct ViewActivity(ObjectSubclass<imp::ViewActivity>)
        @extends View;
}

impl ViewActivity {
    pub fn new(database: Database) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create ViewActivity");

        let self_ = o.get_priv();
        self_
            .activity_model
            .set(ModelActivity::new(database))
            .unwrap();

        self_
            .activities_list_box
            .bind_model(Some(self_.activity_model.get().unwrap()), |o| {
                let row = ActivityRow::new();
                row.set_activity(o.clone().downcast::<Activity>().unwrap());
                row.upcast()
            });

        o
    }

    /// Reload the [ModelActivity]'s data and refresh the list of activities
    pub async fn update(&self) {
        let activity_model = self.get_priv().activity_model.get().unwrap();

        if let Err(e) = activity_model.reload(Duration::days(30)).await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Failed to reload activity data: {}",
                e
            );
        }

        if !activity_model.is_empty() {
            self.upcast_ref::<View>()
                .get_stack()
                .set_visible_child_name("data_page");
        }
    }

    fn get_priv(&self) -> &imp::ViewActivity {
        imp::ViewActivity::from_instance(self)
    }
}
