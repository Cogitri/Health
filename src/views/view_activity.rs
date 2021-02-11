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

use crate::{core::Database, model::ModelActivity, views::View};
use glib::subclass::types::ObjectSubclass;

mod imp {
    use crate::{
        core::Settings,
        model::{Activity, ModelActivity},
        views::View,
        widgets::ActivityRow,
    };
    use chrono::Duration;
    use glib::{subclass, Cast};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_view.ui")]
    pub struct ViewActivity {
        settings: Settings,
        activity_model: OnceCell<ModelActivity>,
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

    impl ViewActivity {
        pub fn set_activity_model(&self, model: ModelActivity) {
            self.activity_model.set(model).unwrap();

            self.activities_list_box
                .bind_model(Some(self.activity_model.get().unwrap()), |o| {
                    let row = ActivityRow::new();
                    row.set_activity(o.clone().downcast::<Activity>().unwrap());
                    row.upcast()
                });
        }

        pub async fn update(&self, obj: &super::ViewActivity) {
            let activity_model = self.activity_model.get().unwrap();

            if let Err(e) = activity_model.reload(Duration::days(30)).await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Failed to reload activity data: {}",
                    e
                );
            }

            if !activity_model.is_empty() {
                obj.upcast_ref::<View>()
                    .get_stack()
                    .set_visible_child_name("data_page");
            }
        }
    }
}

glib::wrapper! {
    pub struct ViewActivity(ObjectSubclass<imp::ViewActivity>)
        @extends View;
}

impl ViewActivity {
    pub fn new(database: Database) -> Self {
        let o = glib::Object::new(&[]).expect("Failed to create ViewActivity");

        imp::ViewActivity::from_instance(&o).set_activity_model(ModelActivity::new(database));

        o
    }

    pub async fn update(&self) {
        imp::ViewActivity::from_instance(self).update(self).await;
    }
}
