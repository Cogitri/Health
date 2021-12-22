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
    use gtk::{glib, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/add_view.ui")]
    pub struct AddView {
        pub icon_name: OnceCell<String>,
        pub view_title: OnceCell<String>,
        #[template_child]
        pub main_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddView {
        const NAME: &'static str = "HealthAddView";
        type ParentType = adw::Bin;
        type Type = super::AddView;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AddView {
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
    impl WidgetImpl for AddView {}
    impl BinImpl for AddView {}
}

glib::wrapper! {
    /// [AddView] is a toplevel container that is implemented by all other views of Health.
    pub struct AddView(ObjectSubclass<imp::AddView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AddView {
    pub fn new(icon_name: &str, view_title: &str) -> Self {
        glib::Object::new(&[("icon-name", &icon_name), ("view-title", &view_title)])
            .expect("Failed to create AddView")
    }
}

pub trait AddViewExt {
    fn icon_name(&self) -> String;
    fn view_title(&self) -> String;
}

impl<O: IsA<AddView>> AddViewExt for O {
    fn icon_name(&self) -> String {
        self.property("icon-name")
    }

    fn view_title(&self) -> String {
        self.property("view-title")
    }
}

unsafe impl<T: BinImpl> IsSubclassable<T> for AddView {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::Bin as IsSubclassable<T>>::class_init(class.upcast_ref_mut());
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::Bin as IsSubclassable<T>>::instance_init(instance);
    }
}

#[cfg(test)]
mod test {
    use super::AddView;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        AddView::new("test", "test");
    }
}
