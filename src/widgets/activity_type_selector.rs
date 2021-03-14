/* activity_type_selector.rs
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

use crate::model::ActivityInfo;
use gio::prelude::*;
use glib::subclass::types::ObjectSubclass;

mod imp {
    use crate::{
        core::Settings,
        model::{ActivityInfo, ActivityType, ActivityTypeRowData},
        widgets::ActivityTypeRow,
    };
    use glib::{
        g_warning,
        subclass::{self, Signal},
    };
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use num_traits::cast::FromPrimitive;
    use std::{cell::RefCell, convert::TryFrom};

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_type_selector.ui")]
    pub struct ActivityTypeSelector {
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

    impl ObjectSubclass for ActivityTypeSelector {
        const NAME: &'static str = "HealthActivityTypeSelector";
        type ParentType = gtk::Popover;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ActivityTypeSelector;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                activity_types_model: gio::ListStore::new(ActivityTypeRowData::static_type()),
                recent_activity_types_model: gio::ListStore::new(ActivityTypeRowData::static_type()),
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

    impl ObjectImpl for ActivityTypeSelector {
        fn constructed(&self, obj: &Self::Type) {
            let recent_activity_types = Settings::get_instance().get_recent_activity_types();

            if !recent_activity_types.is_empty() {
                self.recents_box.set_visible(true);

                for activity in recent_activity_types.iter().rev() {
                    if let Ok(info) = ActivityInfo::try_from(activity.as_str()) {
                        self.recent_activity_types_model
                            .append(&ActivityTypeRowData::new(info.id, &info.name));
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
                        .append(&ActivityTypeRowData::new(info.id, &info.name));
                }

                i += 1;
            }

            let create_list_box_row = glib::clone!(@weak obj => move |o: &glib::Object| {
                let data = o.downcast_ref::<ActivityTypeRowData>().unwrap();
                let selected_activity = ActivityTypeSelector::from_instance(&obj).selected_activity.borrow();
                ActivityTypeRow::new(&data, data.get_label() == selected_activity.name)
                    .upcast::<gtk::Widget>()

            });
            self.activity_types_list_box.bind_model(
                Some(&self.recent_activity_types_model),
                create_list_box_row.clone(),
            );
            self.recent_activity_types_list_box
                .bind_model(Some(&self.activity_types_model), create_list_box_row);

            let activated_list_box_row = glib::clone!(@weak obj => move |b: &gtk::ListBox, r: &gtk::ListBoxRow| {
                let row = r.downcast_ref::<ActivityTypeRow>().unwrap();

                if let Ok(info) = ActivityInfo::try_from(row.get_id()) {
                    let self_ = ActivityTypeSelector::from_instance(&obj);
                    obj.set_selected_activity(info);
                    let mut i = 0;
                    let selected_activity = self_.selected_activity.borrow();

                    while let Some(row) = b.get_row_at_index(i) {
                        let cast = row.downcast::<ActivityTypeRow>().unwrap();
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
                vec![Signal::builder("activity-selected", &[], glib::Type::UNIT.into()).build()]
            });

            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for ActivityTypeSelector {}
    impl PopoverImpl for ActivityTypeSelector {}
}

glib::wrapper! {
    /// A widget for selecting an [ActivityType](crate::model::ActivityType) (e.g. for adding a new activity).
    pub struct ActivityTypeSelector(ObjectSubclass<imp::ActivityTypeSelector>)
        @extends gtk::Widget, gtk::Popover;
}

impl ActivityTypeSelector {
    pub fn get_selected_activity(&self) -> ActivityInfo {
        self.get_priv().selected_activity.borrow().clone()
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

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ActivityTypeSelector")
    }

    fn get_priv(&self) -> &imp::ActivityTypeSelector {
        imp::ActivityTypeSelector::from_instance(self)
    }

    fn set_selected_activity(&self, val: ActivityInfo) {
        self.get_priv().selected_activity.replace(val);
        self.emit_by_name("activity-selected", &[]).unwrap();
    }
}
