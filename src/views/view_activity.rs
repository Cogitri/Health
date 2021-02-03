use crate::{core::Database, model::ModelActivity, views::View};
use gdk::subclass::prelude::*;

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
        pub activities_list_box: gtk::ListBox,
        clamp: adw::Clamp,
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
            let activities_list_box = gtk::ListBoxBuilder::new()
                .can_focus(false)
                .selection_mode(gtk::SelectionMode::None)
                .build();
            activities_list_box.add_css_class("content");
            let clamp_builder = adw::ClampBuilder::new()
                .maximum_size(800)
                .tightening_threshold(600)
                .valign(gtk::Align::Center)
                .vexpand(true)
                .hexpand(true)
                .margin_end(6)
                .margin_bottom(6)
                .margin_start(6)
                .margin_top(6)
                .child(&activities_list_box);

            Self {
                settings: Settings::new(),
                activity_model: OnceCell::new(),
                activities_list_box,
                clamp: clamp_builder.build(),
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

    impl ObjectImpl for ViewActivity {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let scrolled_window = obj.upcast_ref::<View>().get_scrolled_window();
            scrolled_window.set_child(Some(&self.clamp));
            scrolled_window.set_property_vscrollbar_policy(gtk::PolicyType::Automatic);
        }
    }

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