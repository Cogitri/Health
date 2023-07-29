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

use crate::{
    core::{Database, Settings},
    model::{ActivityInfo, ActivityInfoBoxed, ActivityType, ActivityTypeRowData},
    widgets::ActivityTypeRow,
};
use gtk::{
    gio::{prelude::*, subclass::prelude::*},
    glib::{self, g_warning, SignalHandlerId},
    prelude::*,
};
use num_traits::cast::FromPrimitive;
use std::convert::TryFrom;

mod imp {
    use crate::{
        model::{ActivityInfo, ActivityInfoBoxed, ActivityType, ActivityTypeRowData},
        widgets::ActivityTypeRow,
    };
    use gtk::{
        gio, glib,
        {prelude::*, subclass::prelude::*, CompositeTemplate},
    };
    use std::cell::RefCell;

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
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityTypeSelector {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            gtk_macros::spawn!(glib::clone!(@weak obj => async move {
                obj.load_recent_activities().await;
            }));

            let create_list_box_row = glib::clone!(@weak obj => @default-panic, move |o: &glib::Object| {
                let data = o.downcast_ref::<ActivityTypeRowData>().unwrap();
                let selected_activity = obj.imp().selected_activity.borrow();
                ActivityTypeRow::new(data, data.label() == selected_activity.name)
                    .upcast::<gtk::Widget>()

            });
            self.activity_types_list_box.bind_model(
                Some(&self.activity_types_model),
                create_list_box_row.clone(),
            );
            self.recent_activity_types_list_box
                .bind_model(Some(&self.recent_activity_types_model), create_list_box_row);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::builder::<ActivityInfoBoxed>("selected-activity").build(),
                ]
            });
            &PROPERTIES
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-activity" => {
                    ActivityInfoBoxed(self.selected_activity.borrow().clone()).to_value()
                }
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
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
        @extends gtk::Widget, gtk::Popover,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
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

    pub async fn load_recent_activities(&self) {
        let imp = self.imp();
        let user_id = i64::from(Settings::instance().active_user_id());
        let user = &Database::instance().user(user_id).await.unwrap();

        let recent_activity_types = user.recent_activity_types().unwrap();

        if !recent_activity_types.is_empty() {
            imp.recents_box.set_visible(true);

            for activity in recent_activity_types.iter().rev() {
                if let Ok(info) = ActivityInfo::try_from(*activity) {
                    imp.recent_activity_types_model
                        .append(&ActivityTypeRowData::new(info.id, &info.name));
                } else {
                    let name = activity.as_ref();
                    g_warning!(crate::config::LOG_DOMAIN, "Unknown activity {name}!");
                }
            }
            let last_activity = recent_activity_types.last().unwrap();
            if let Ok(info) = ActivityInfo::try_from(*last_activity) {
                imp.selected_activity.replace(info);
            } else {
                let name = last_activity.as_ref();
                g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Unknown Activity {name}, falling back to walking.",
                );
            }
        }

        let mut i = 0;
        while let Some(a) = ActivityType::from_i32(i) {
            let info = ActivityInfo::from(a);
            if !recent_activity_types.contains(&info.activity_type) {
                imp.activity_types_model
                    .append(&ActivityTypeRowData::new(info.id, &info.name));
            }

            i += 1;
        }
    }

    /// Get the currently selected [ActivityInfo].
    pub fn selected_activity(&self) -> ActivityInfo {
        self.property::<ActivityInfoBoxed>("selected-activity").0
    }

    /// Create a new [ActivityTypeSelector].
    pub fn new() -> Self {
        glib::Object::new()
    }

    #[template_callback]
    fn activated_list_box_row(&self, row: gtk::ListBoxRow, list_box: gtk::ListBox) {
        let row = row.downcast_ref::<ActivityTypeRow>().unwrap();
        let imp = self.imp();

        if let Ok(info) = ActivityInfo::try_from(row.id().as_str()) {
            self.set_selected_activity(info);
            let mut i = 0;
            let selected_activity = self.imp().selected_activity.borrow();

            while let Some(row) = list_box.row_at_index(i) {
                let cast = row.downcast::<ActivityTypeRow>().unwrap();
                cast.set_selected(cast.label() == selected_activity.name);
                i += 1;
            }

            let other_box = if list_box == imp.recent_activity_types_list_box.get() {
                imp.activity_types_list_box.get()
            } else {
                imp.recent_activity_types_list_box.get()
            };

            while let Some(row) = other_box.row_at_index(i) {
                let cast = row.downcast::<ActivityTypeRow>().unwrap();
                cast.set_selected(false);
                i += 1;
            }

            self.popdown();
        } else {
            glib::g_warning!(crate::config::LOG_DOMAIN, "Unknown Activity {}", row.id());
        }
    }

    fn set_selected_activity(&self, val: ActivityInfo) {
        self.set_property("selected-activity", ActivityInfoBoxed(val))
    }
}

#[cfg(test)]
mod test {
    use super::ActivityTypeSelector;
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        ActivityTypeSelector::new();
    }
}
