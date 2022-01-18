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
use gtk::{
    gio::{self, prelude::*},
    glib::{self, subclass::prelude::*},
};
use std::convert::TryInto;
mod imp {
    use crate::{
        core::{Database, Settings},
        model::Activity,
    };
    use gtk::subclass::prelude::*;
    use gtk::{
        gio,
        glib::{self, subclass, Cast, StaticType},
    };
    use std::{
        cell::RefCell,
        convert::{TryFrom, TryInto},
    };

    #[derive(Debug, Default)]
    pub struct ModelActivityMut {
        pub vec: Vec<Activity>,
    }

    #[derive(Debug, Default)]
    pub struct ModelActivity {
        pub database: Database,
        pub inner: RefCell<ModelActivityMut>,
        pub settings: Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModelActivity {
        const NAME: &'static str = "HealthModelActivity";
        type ParentType = glib::Object;
        type Instance = subclass::basic::InstanceStruct<Self>;
        type Class = subclass::basic::ClassStruct<Self>;
        type Type = super::ModelActivity;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for ModelActivity {}
    impl ListModelImpl for ModelActivity {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Activity::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.inner.borrow().vec.len().try_into().unwrap()
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.inner
                .borrow()
                .vec
                .get(usize::try_from(position).unwrap())
                .map(|o| o.clone().upcast())
        }
    }
}

glib::wrapper! {
    /// An implementation of [gio::ListModel] that stores [Activity](crate::model::Activity)s.
    /// Can be used with [ActivityView](crate::views::ViewActivity) to display past activities.
    pub struct ModelActivity(ObjectSubclass<imp::ModelActivity>) @implements gio::ListModel;
}

impl Default for ModelActivity {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelActivity {
    pub fn is_empty(&self) -> bool {
        self.imp().inner.borrow().vec.is_empty()
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ModelActivity")
    }

    /// Reload the data from the Tracker Database.
    ///
    /// # Arguments
    /// * `duration` - How far in the past the data should reach back.
    ///
    /// # Returns start date of viewing period (None for ViewPeriod::All)
    /// Returns an error if querying the DB fails.
    pub async fn reload(&self) -> Result<()> {
        let self_ = self.imp();
        let previous_size = { self_.inner.borrow().vec.len() };

        let new_vec = self_.database.activities(None).await?;
        {
            self_.inner.borrow_mut().vec = new_vec;
        }
        self.items_changed(
            0,
            previous_size.try_into().unwrap(),
            self_.inner.borrow().vec.len().try_into().unwrap(),
        );
        Ok(())
    }

    pub async fn activity_present(&self) -> bool {
        self.imp()
            .database
            .num_activities()
            .await
            .map(|x| x != 0)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::ModelActivity;

    #[test]
    fn new() {
        ModelActivity::new();
    }
}
