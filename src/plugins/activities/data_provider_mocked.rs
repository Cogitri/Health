/* model_activity.rs
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

use anyhow::Result;
use gtk::{gio, glib};
mod imp {
    use crate::{
        model::{Activity, ActivityType},
        prelude::*,
    };
    use gtk::subclass::prelude::*;
    use gtk::{
        gio,
        glib::{self, subclass, Cast, StaticType},
        prelude::*,
    };
    use once_cell::unsync::OnceCell;
    use std::convert::{TryFrom, TryInto};

    #[derive(Debug, Default)]
    pub struct ModelActivityMocked {
        pub vec: OnceCell<Vec<Activity>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModelActivityMocked {
        const NAME: &'static str = "HealthModelActivityMocked";
        type ParentType = glib::Object;
        type Instance = subclass::basic::InstanceStruct<Self>;
        type Class = subclass::basic::ClassStruct<Self>;
        type Type = super::ModelActivityMocked;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ModelActivityMocked {
        fn constructed(&self) {
            let obj = self.obj();
            let now = glib::DateTime::local();
            let a = Activity::new();
            a.set_activity_type(ActivityType::Walking);
            a.set_duration(glib::TimeSpan::from_minutes(75));
            a.set_date(now.clone());
            a.set_steps(Some(5000));
            a.set_calories_burned(Some(200));

            let b = Activity::new();
            b.set_activity_type(ActivityType::Walking);
            b.set_duration(glib::TimeSpan::from_minutes(23));
            b.set_date(now);
            b.set_steps(Some(2300));
            b.set_calories_burned(Some(75));

            self.vec.set(vec![a, b]).unwrap();
            obj.items_changed(0, 0, self.vec.get().unwrap().len().try_into().unwrap());
        }
    }

    impl ListModelImpl for ModelActivityMocked {
        fn item_type(&self) -> glib::Type {
            Activity::static_type()
        }

        fn n_items(&self) -> u32 {
            self.vec.get().unwrap().len().try_into().unwrap()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.vec
                .get()
                .unwrap()
                .get(usize::try_from(position).unwrap())
                .map(|o| o.clone().upcast())
        }
    }
}

glib::wrapper! {
    /// An implementation of [gio::ListModel] that stores a few static, mocked [Activity](crate::model::Activity)s.
    /// Can be used with [ActivityView](crate::views::ViewActivity) to display past activities.
    pub struct ModelActivityMocked(ObjectSubclass<imp::ModelActivityMocked>) @implements gio::ListModel;
}

impl Default for ModelActivityMocked {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelActivityMocked {
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub async fn activity_present(&self) -> bool {
        true
    }

    pub async fn reload(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::ModelActivityMocked;

    #[test]
    fn new() {
        ModelActivityMocked::new();
    }
}
