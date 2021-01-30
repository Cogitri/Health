use crate::model::ActivityInfo;
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use crate::{
        core::HealthSettings,
        model::{ActivityType, HealthActivityTypeRowData},
        widgets::HealthActivityTypeRow,
    };
    use glib::{
        g_warning,
        subclass::{self, Signal},
    };
    use gtk::subclass::prelude::*;
    use num_traits::cast::FromPrimitive;
    use std::{cell::RefCell, convert::TryFrom};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_type_selector.ui")]
    pub struct HealthActivityTypeSelector {
        pub activity_types_model: gio::ListStore,
        pub recent_activity_types_model: gio::ListStore,
        pub selected_activity: RefCell<ActivityInfo>,
        #[template_child]
        pub activity_types_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub recent_activity_types_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub recents_box: TemplateChild<gtk::Box>,
    }

    impl ObjectSubclass for HealthActivityTypeSelector {
        const NAME: &'static str = "HealthActivityTypeSelector";
        type ParentType = gtk::Popover;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthActivityTypeSelector;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                activity_types_model: gio::ListStore::new(HealthActivityTypeRowData::static_type()),
                recent_activity_types_model: gio::ListStore::new(
                    HealthActivityTypeRowData::static_type(),
                ),
                selected_activity: RefCell::new(ActivityInfo::from(ActivityType::Walking)),
                activity_types_list_box: TemplateChild::default(),
                recent_activity_types_list_box: TemplateChild::default(),
                recents_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HealthActivityTypeSelector {
        fn constructed(&self, obj: &Self::Type) {
            let recent_activity_types = HealthSettings::new().get_recent_activity_types();

            if !recent_activity_types.is_empty() {
                self.recents_box.set_visible(true);

                for activity in recent_activity_types.iter().rev() {
                    if let Ok(info) = ActivityInfo::try_from(activity.as_str()) {
                        self.recent_activity_types_model
                            .append(&HealthActivityTypeRowData::new(info.id, &info.name));
                    } else {
                        g_warning!(crate::config::LOG_DOMAIN, "Unknown activity {}!", activity);
                    }
                }
                let last_activity = recent_activity_types.last().unwrap().as_str();
                if let Ok(info) = ActivityInfo::try_from(last_activity) {
                    self.selected_activity.replace(info);
                } else {
                    g_warning!(
                        crate::config::LOG_DOMAIN,
                        "Unknown Activity {}, falling back to walking.",
                        last_activity
                    );
                }
            }

            let mut i = 0;
            while let Some(a) = ActivityType::from_i32(i) {
                let info = ActivityInfo::from(a);
                if !recent_activity_types.contains(&info.id.to_string()) {
                    self.activity_types_model
                        .append(&HealthActivityTypeRowData::new(info.id, &info.name));
                }

                i += 1;
            }

            let create_list_box_row = glib::clone!(@weak obj => move |o: &glib::Object| {
                let data = o.downcast_ref::<HealthActivityTypeRowData>().unwrap();
                HealthActivityTypeRow::new(&data, data.get_label() == imp::HealthActivityTypeSelector::from_instance(&obj).selected_activity.borrow().name)
                    .upcast::<gtk::Widget>()

            });
            self.activity_types_list_box.bind_model(
                Some(&self.recent_activity_types_model),
                create_list_box_row.clone(),
            );
            self.recent_activity_types_list_box
                .bind_model(Some(&self.activity_types_model), create_list_box_row);

            let activated_list_box_row = glib::clone!(@weak obj => move |b: &gtk::ListBox, r: &gtk::ListBoxRow| {
                let row = r.downcast_ref::<HealthActivityTypeRow>().unwrap();

                if let Ok(info) = ActivityInfo::try_from(row.get_id()) {
                    let self_ = imp::HealthActivityTypeSelector::from_instance(&obj);
                    self_.set_selected_activity(&obj, info);
                    let mut i = 0;
                    let selected_activity = self_.selected_activity.borrow();

                    while let Some(row) = b.get_row_at_index(i) {
                        let cast = row.downcast::<HealthActivityTypeRow>().unwrap();
                        cast.set_selected (cast.get_label() == selected_activity.name);
                        i += 1;
                    }

                    obj.popdown();
                }  else {
                    g_warning!(
                        crate::config::LOG_DOMAIN,
                        "Unknown Activity {}",
                        row.get_id()
                    );
                }
            });

            self.activity_types_list_box
                .connect_row_activated(activated_list_box_row.clone());
            self.recent_activity_types_list_box
                .connect_row_activated(activated_list_box_row);
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("activity-selected", &[], glib::Type::Unit).build()]
            });

            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for HealthActivityTypeSelector {}
    impl PopoverImpl for HealthActivityTypeSelector {}

    impl HealthActivityTypeSelector {
        pub fn get_selected_activity(&self) -> ActivityInfo {
            self.selected_activity.borrow().clone()
        }

        pub fn set_selected_activity(
            &self,
            obj: &super::HealthActivityTypeSelector,
            val: ActivityInfo,
        ) {
            self.selected_activity.replace(val);
            obj.emit("activity-selected", &[]).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct HealthActivityTypeSelector(ObjectSubclass<imp::HealthActivityTypeSelector>)
        @extends gtk::Widget, gtk::Popover;
}

impl HealthActivityTypeSelector {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create HealthActivityTypeSelector")
    }

    pub fn get_selected_activity(&self) -> ActivityInfo {
        imp::HealthActivityTypeSelector::from_instance(self).get_selected_activity()
    }

    pub fn connect_activity_selected<F: Fn() + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("activity-selected", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }
}
