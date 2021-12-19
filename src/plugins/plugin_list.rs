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

use crate::{plugins::Plugin, RefIter};
use gtk::{
    gio::{self, prelude::*},
    glib::{self, subclass::prelude::*},
};
use std::{cell::Ref, convert::TryInto};

mod imp {
    use crate::plugins::{Plugin, PluginObject};
    use gtk::subclass::prelude::*;
    use gtk::{
        gio,
        glib::{self, Cast, StaticType},
    };
    use std::{
        cell::RefCell,
        convert::{TryFrom, TryInto},
    };

    #[derive(Debug, Default)]
    pub struct PluginList {
        pub vec: RefCell<Vec<Box<dyn Plugin>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginList {
        const NAME: &'static str = "HealthPluginList";
        type ParentType = glib::Object;
        type Type = super::PluginList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for PluginList {}
    impl ListModelImpl for PluginList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            PluginObject::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.vec.borrow().len().try_into().unwrap()
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.vec
                .borrow()
                .get(usize::try_from(position).unwrap())
                .map(|o| PluginObject::new(o.clone()).upcast())
        }
    }
}

glib::wrapper! {
    /// An implementation of [gio::ListModel] that stores [Plugin](crate::plugin::Plugin)s.
    pub struct PluginList(ObjectSubclass<imp::PluginList>) @implements gio::ListModel;
}

impl Default for PluginList {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl PluginList {
    pub fn contains(&self, plugin_name: &str) -> bool {
        self.imp()
            .vec
            .borrow()
            .iter()
            .any(|p| p.name() == plugin_name)
    }

    pub fn is_empty(&self) -> bool {
        self.imp().vec.borrow().is_empty()
    }
    pub fn first(&self) -> Option<Box<dyn Plugin>> {
        self.imp().vec.borrow().first().cloned()
    }

    pub fn get(&self, index: usize) -> Option<Box<dyn Plugin>> {
        self.imp().vec.borrow().get(index).cloned()
    }

    pub fn iter(&self) -> RefIter<Box<dyn Plugin>> {
        RefIter::new(Ref::map(self.imp().vec.borrow(), |v| &v[..]))
    }

    pub fn last(&self) -> Option<Box<dyn Plugin>> {
        self.imp().vec.borrow().last().cloned()
    }

    pub fn len(&self) -> usize {
        self.imp().vec.borrow().len()
    }

    pub fn new(plugin_list: Vec<Box<dyn Plugin>>) -> Self {
        let o: Self = glib::Object::new(&[]).expect("Failed to create PluginList");
        o.imp().vec.replace(plugin_list);
        o
    }

    pub fn push(&self, plugin: Box<dyn Plugin>) {
        let len;
        {
            let mut vec = self.imp().vec.borrow_mut();
            len = vec.len();
            vec.push(plugin);
        }
        self.items_changed(len.try_into().unwrap(), 0, 1);
    }

    pub fn remove(&self, plugin_name: &str) -> Option<Box<dyn Plugin>> {
        let mut changed_position: Option<usize> = None;
        let mut ret: Option<Box<dyn Plugin>> = None;

        {
            let mut vec = self.imp().vec.borrow_mut();
            if let Some(f) = vec.iter().position(|x| x.name() == plugin_name) {
                ret = Some(vec.remove(f));
                changed_position = Some(f);
            }
        }
        if let Some(pos) = changed_position {
            self.items_changed(pos.try_into().unwrap(), 1, 0);
        }

        ret
    }

    fn imp(&self) -> &imp::PluginList {
        imp::PluginList::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use super::PluginList;

    #[test]
    fn new() {
        PluginList::new(Vec::new());
    }
}
