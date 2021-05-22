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

use crate::{model::ActivityInfo, ActivityInfoBoxed};
use gio::prelude::*;
use gio::subclass::prelude::*;
use glib::SignalHandlerId;

mod imp {
    use crate::{
        core::settings::prelude::*,
        model::{ActivityInfo, ActivityType, ActivityTypeRowData},
        widgets::ActivityTypeRow,
        ActivityInfoBoxed,
    };
    use gio::Settings;
    use glib::g_warning;
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

    #[glib::object_subclass]
    impl ObjectSubclass for ActivityTypeSelector {
        const NAME: &'static str = "HealthActivityTypeSelector";
        type ParentType = gtk::Popover;
        type Type = super::ActivityTypeSelector;

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

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityTypeSelector {
        fn constructed(&self, obj: &Self::Type) {
            let recent_activity_types = Settings::instance().recent_activity_types();

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

            let create_list_box_row = glib::clone!(@weak obj => @default-panic, move |o: &glib::Object| {
                let data = o.downcast_ref::<ActivityTypeRowData>().unwrap();
                let selected_activity = ActivityTypeSelector::from_instance(&obj).selected_activity.borrow();
                ActivityTypeRow::new(&data, data.label() == selected_activity.name)
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

                if let Ok(info) = ActivityInfo::try_from(row.id()) {
                    let self_ = ActivityTypeSelector::from_instance(&obj);
                    obj.set_selected_activity(info);
                    let mut i = 0;
                    let selected_activity = self_.selected_activity.borrow();

                    while let Some(row) = b.row_at_index(i) {
                        let cast = row.downcast::<ActivityTypeRow>().unwrap();
                        cast.set_selected (cast.label() == selected_activity.name);
                        i += 1;
                    }

                    obj.popdown();
                }  else {
                    g_warning!(
                        crate::config::LOG_DOMAIN,
                        "Unknown Activity {}",
                        row.id()
                    );
                }
            });

            self.activity_types_list_box
                .connect_row_activated(activated_list_box_row.clone());
            self.recent_activity_types_list_box
                .connect_row_activated(activated_list_box_row);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boxed(
                    "selected-activity",
                    "selected-activity",
                    "selected-activity",
                    ActivityInfoBoxed::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
            &PROPERTIES
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-activity" => {
                    ActivityInfoBoxed(self.selected_activity.borrow().clone()).to_value()
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "selected-activity" => {
                    self.selected_activity
                        .replace(value.get::<ActivityInfoBoxed>().unwrap().0);
                }
                _ => unimplemented!(),
            }
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
    /// Connect to a new activity being selected.
    ///
    /// # Arguments
    /// * `callback` - The callback to call once the ::notify signal is emitted.
    ///
    /// # Returns
    /// The [glib::SignalHandlerId] to disconnect the signal later on.
    pub fn connect_selected_activity_notify<F: Fn(&Self) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        self.connect_notify_local(Some("selected-activity"), move |s, _| f(s))
    }

    /// Get the currently selected [ActivityInfo].
    pub fn selected_activity(&self) -> ActivityInfo {
        self.property("selected-activity")
            .unwrap()
            .get::<ActivityInfoBoxed>()
            .unwrap()
            .0
    }

    /// Create a new [ActivityTypeSelector].
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ActivityTypeSelector")
    }

    #[allow(dead_code)]
    fn imp(&self) -> &imp::ActivityTypeSelector {
        imp::ActivityTypeSelector::from_instance(self)
    }

    fn set_selected_activity(&self, val: ActivityInfo) {
        self.set_property("selected-activity", ActivityInfoBoxed(val))
            .unwrap();
    }
}
