/* add_view.rs
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

use adw::subclass::prelude::*;
use gtk::glib::{self, prelude::*};
mod imp {
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{glib, subclass::prelude::*};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub struct ViewAdd {
        pub icon_name: OnceCell<String>,
        pub view_title: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewAdd {
        const NAME: &'static str = "HealthViewAdd";
        type ParentType = adw::Bin;
        type Type = super::ViewAdd;
    }

    impl ObjectImpl for ViewAdd {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::new(
                        "icon-name",
                        "icon-name",
                        "icon-name",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecString::new(
                        "view-title",
                        "view-title",
                        "view-title",
                        None,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "icon-name" => self.icon_name.set(value.get().unwrap()).unwrap(),
                "view-title" => self.view_title.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "icon-name" => self.icon_name.get().unwrap().to_value(),
                "view-title" => self.view_title.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ViewAdd {}
    impl BinImpl for ViewAdd {}
}

glib::wrapper! {
    /// [ViewAdd] is a toplevel container that is implemented by all other views of Health.
    pub struct ViewAdd(ObjectSubclass<imp::ViewAdd>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewAdd {
    pub fn new(icon_name: &str, view_title: &str) -> Self {
        glib::Object::new(&[("icon-name", &icon_name), ("view-title", &view_title)])
            .expect("Failed to create ViewAdd")
    }
}

pub trait ViewAddExt {
    fn icon_name(&self) -> String;
    fn view_title(&self) -> String;
}

impl<O: IsA<ViewAdd>> ViewAddExt for O {
    fn icon_name(&self) -> String {
        self.property("icon-name")
    }

    fn view_title(&self) -> String {
        self.property("view-title")
    }
}

unsafe impl<T: BinImpl> IsSubclassable<T> for ViewAdd {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::Bin as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::Bin as IsSubclassable<T>>::instance_init(instance);
    }
}

#[cfg(test)]
mod test {
    use super::ViewAdd;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        ViewAdd::new("test", "test");
    }
}
