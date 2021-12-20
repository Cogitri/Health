/* PluginDetails.rs
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
use anyhow::Result;
use gtk::{
    gio,
    glib::{self, prelude::*},
};
use std::{future::Future, pin::Pin};

pub type PinnedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + 'static>>;

mod imp {
    use super::PinnedResultFuture;
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{gio, glib, subclass::prelude::*, CompositeTemplate};
    use std::cell::Cell;

    #[repr(C)]
    pub struct PluginDetailsClass {
        pub parent_class: adw::ffi::AdwBinClass,
        pub update_actual: fn(&super::PluginDetails) -> PinnedResultFuture,
    }

    unsafe impl ClassStruct for PluginDetailsClass {
        type Type = PluginDetails;
    }

    impl std::ops::Deref for PluginDetailsClass {
        type Target = glib::Class<glib::Object>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const Self).cast::<Self::Target>() }
        }
    }

    impl std::ops::DerefMut for PluginDetailsClass {
        fn deref_mut(&mut self) -> &mut glib::Class<glib::Object> {
            unsafe { &mut *(self as *mut Self).cast::<glib::Class<glib::Object>>() }
        }
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/plugins/details.ui")]
    pub struct PluginDetails {
        pub is_mocked: Cell<bool>,
        #[template_child]
        pub empty_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub empty_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub filled_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub filled_subtitle_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub main_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
    }

    // Virtual method default implementation trampolines
    fn update_actual_default_trampoline(this: &super::PluginDetails) -> PinnedResultFuture {
        PluginDetails::from_instance(this).update_actual(this)
    }

    pub(super) fn plugin_details_update_actual(this: &super::PluginDetails) -> PinnedResultFuture {
        let klass = this.class();

        (klass.as_ref().update_actual)(this)
    }

    impl PluginDetails {
        fn update_actual(&self, obj: &super::PluginDetails) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginDetails {
        const NAME: &'static str = "HealthPluginDetails";
        type ParentType = adw::Bin;
        type Type = super::PluginDetails;
        type Class = PluginDetailsClass;

        fn class_init(klass: &mut Self::Class) {
            klass.update_actual = update_actual_default_trampoline;
            klass.set_layout_manager_type::<gtk::BinLayout>();

            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PluginDetails {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::new(
                        "content-widget",
                        "content-widget",
                        "content-widget",
                        gtk::Widget::static_type(),
                        glib::ParamFlags::WRITABLE,
                    ),
                    glib::ParamSpecString::new(
                        "empty-label",
                        "empty-label",
                        "empty-label",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "empty-icon-name",
                        "empty-icon-name",
                        "empty-icon-name",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "filled-title",
                        "filled-title",
                        "filled-title",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "filled-subtitle",
                        "filled-subtitle",
                        "filled-subtitle",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoolean::new(
                        "is-mocked",
                        "is-mocked",
                        "is-mocked",
                        false,
                        glib::ParamFlags::READWRITE,
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
                "content-widget" => self.main_box.append(&value.get::<gtk::Widget>().unwrap()),
                "empty-label" => self.empty_label.set_label(value.get::<&str>().unwrap()),
                "empty-icon-name" => self.empty_icon.set_icon_name(value.get().unwrap()),
                "filled-title" => self
                    .filled_title_label
                    .set_label(value.get::<&str>().unwrap()),
                "filled-subtitle" => self
                    .filled_subtitle_label
                    .set_label(value.get::<&str>().unwrap_or("")),
                "is-mocked" => self.is_mocked.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "empty-label" => self.empty_label.label().to_value(),
                "empty-icon-name" => self.empty_icon.icon_name().to_value(),
                "filled-title" => self.filled_title_label.label().to_value(),
                "filled-subtitle" => self.filled_subtitle_label.label().to_value(),
                "is-mocked" => self.is_mocked.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for PluginDetails {}
    impl BinImpl for PluginDetails {}
}

glib::wrapper! {
    /// [PluginDetails] is a toplevel container that is implemented by all other PluginDetailss of Health.
    pub struct PluginDetails(ObjectSubclass<imp::PluginDetails>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginDetails {
    fn imp(&self) -> &imp::PluginDetails {
        imp::PluginDetails::from_instance(self)
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create PluginDetails")
    }
}

pub trait PluginDetailsExt {
    fn empty_icon_name(&self) -> String;
    fn empty_label(&self) -> String;
    fn filled_subtitle(&self) -> String;
    fn filled_title(&self) -> String;
    fn is_mocked(&self) -> bool;

    fn set_empty_icon_name(&self, val: &str);
    fn set_empty_label(&self, val: &str);
    fn set_filled_subtitle(&self, val: &str);
    fn set_filled_title(&self, val: &str);
    fn set_is_mocked(&self, val: bool);

    fn switch_to_data_page(&self);
    fn switch_to_empty_page(&self);

    fn update(&self) -> PinnedResultFuture;
    fn update_actual(&self) -> PinnedResultFuture;
}

impl<O: IsA<PluginDetails>> PluginDetailsExt for O {
    fn empty_icon_name(&self) -> String {
        self.property("empty-icon-name")
    }
    fn empty_label(&self) -> String {
        self.property("empty-label")
    }
    fn filled_subtitle(&self) -> String {
        self.property("filled-subtitle")
    }
    fn filled_title(&self) -> String {
        self.property("filled-title")
    }
    fn is_mocked(&self) -> bool {
        self.property("is-mocked")
    }

    fn set_empty_icon_name(&self, val: &str) {
        self.set_property("empty-icon-name", val);
    }
    fn set_empty_label(&self, val: &str) {
        self.set_property("empty-label", val);
    }
    fn set_filled_subtitle(&self, val: &str) {
        self.set_property("filled-subtitle", val);
    }
    fn set_filled_title(&self, val: &str) {
        self.set_property("filled-title", val);
    }
    fn set_is_mocked(&self, val: bool) {
        self.set_property("is-mocked", val);
    }

    fn switch_to_data_page(&self) {
        self.upcast_ref::<PluginDetails>()
            .imp()
            .stack
            .set_visible_child_name("data_page");
    }
    fn switch_to_empty_page(&self) {
        self.upcast_ref::<PluginDetails>()
            .imp()
            .stack
            .set_visible_child_name("empty_page");
    }

    fn update(&self) -> PinnedResultFuture {
        if self.is_mocked() {
            Box::pin(gio::GioFuture::new(self.upcast_ref(), move |_, _, send| {
                send.resolve(Ok(()));
            }))
        } else {
            self.update_actual()
        }
    }

    fn update_actual(&self) -> PinnedResultFuture {
        imp::plugin_details_update_actual(self.upcast_ref())
    }
}

pub trait PluginDetailsImpl: BinImpl + 'static {
    fn update_actual(&self, obj: &PluginDetails) -> PinnedResultFuture {
        self.parent_update_actual(obj)
    }
}

pub trait PluginDetailsImplExt: ObjectSubclass {
    fn parent_update_actual(&self, obj: &PluginDetails) -> PinnedResultFuture;
}

impl<T: PluginDetailsImpl> PluginDetailsImplExt for T {
    fn parent_update_actual(&self, obj: &PluginDetails) -> PinnedResultFuture {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut imp::PluginDetailsClass);
            (parent_class.update_actual)(obj)
        }
    }
}

unsafe impl<T: PluginDetailsImpl> IsSubclassable<T> for PluginDetails {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::Bin as IsSubclassable<T>>::class_init(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.update_actual = update_actual_trampoline::<T>;
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::Bin as IsSubclassable<T>>::instance_init(instance);
    }
}

// Virtual method default implementation trampolines
fn update_actual_trampoline<T: ObjectSubclass>(this: &PluginDetails) -> PinnedResultFuture
where
    T: PluginDetailsImpl,
{
    let imp = T::from_instance(this.dynamic_cast_ref::<T::Type>().unwrap());
    imp.update_actual(this)
}

#[cfg(test)]
mod test {
    use super::PluginDetails;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginDetails::new();
    }
}