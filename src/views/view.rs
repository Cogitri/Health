/* view.rs
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

use crate::properties_setter_getter;
use anyhow::Result;
use gtk::{
    glib::{self, prelude::*},
    subclass::prelude::*,
};
use std::{future::Future, pin::Pin};

pub type PinnedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + 'static>>;
mod imp {
    use super::PinnedResultFuture;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
    use std::cell::RefCell;

    pub type ViewInstance = super::View;

    #[repr(C)]
    pub struct ViewClass {
        pub parent_class: gtk::ffi::GtkWidgetClass,
        pub update: fn(&ViewInstance) -> PinnedResultFuture,
    }

    unsafe impl ClassStruct for ViewClass {
        type Type = View;
    }

    impl std::ops::Deref for ViewClass {
        type Target = glib::Class<glib::Object>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const Self).cast::<Self::Target>() }
        }
    }

    impl std::ops::DerefMut for ViewClass {
        fn deref_mut(&mut self) -> &mut glib::Class<glib::Object> {
            unsafe { &mut *(self as *mut Self).cast::<glib::Class<glib::Object>>() }
        }
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/view.ui")]
    pub struct View {
        #[template_child]
        pub empty_icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub goal_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub main_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub subtitle_empty_view_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub title_empty_view_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        pub view_title: RefCell<String>,
    }

    // Virtual method default implementation trampolines
    fn update_default_trampoline(this: &ViewInstance) -> PinnedResultFuture {
        View::from_instance(this).update(this)
    }

    pub(super) fn view_update(this: &ViewInstance) -> PinnedResultFuture {
        let klass = this.class();

        (klass.as_ref().update)(this)
    }

    impl View {
        fn update(&self, obj: &super::View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for View {
        const NAME: &'static str = "HealthView";
        type ParentType = gtk::Widget;
        type Type = super::View;
        type Class = ViewClass;

        fn class_init(klass: &mut Self::Class) {
            klass.update = update_default_trampoline;
            klass.set_layout_manager_type::<gtk::BinLayout>();

            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl WidgetImpl for View {}

    impl ObjectImpl for View {
        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

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
                        "empty-subtitle",
                        "empty-subtitle",
                        "empty-subtitle",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "icon-name",
                        "icon-name",
                        "icon-name",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "title",
                        "title",
                        "title",
                        None,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecString::new(
                        "view-title",
                        "view-title",
                        "view-title",
                        None,
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
                "empty-subtitle" => self
                    .subtitle_empty_view_label
                    .set_label(value.get::<&str>().unwrap_or("")),
                "icon-name" => self.empty_icon.set_icon_name(value.get().unwrap()),
                "title" => self
                    .title_label
                    .set_label(value.get::<&str>().unwrap_or("")),
                "view-title" => {
                    self.view_title
                        .replace(value.get::<String>().unwrap_or_else(|_| "".to_string()));
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "empty-subtitle" => self.subtitle_empty_view_label.label().to_value(),
                "icon-name" => self.empty_icon.icon_name().to_value(),
                "title" => self.title_label.label().to_value(),
                "view-title" => self.view_title.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// [View] is a toplevel container that is implemented by all other views of Health.
    pub struct View(ObjectSubclass<imp::View>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl View {
    pub fn goal_label(&self) -> gtk::Label {
        self.imp().goal_label.get()
    }

    pub fn stack(&self) -> gtk::Stack {
        self.imp().stack.get()
    }

    fn imp(&self) -> &imp::View {
        imp::View::from_instance(self)
    }

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create View")
    }

    properties_setter_getter!("empty-subtitle", String);
    properties_setter_getter!("icon-name", String);
    properties_setter_getter!("title", String);
    properties_setter_getter!("view-title", String);
}

pub trait ViewExt {
    fn update(&self) -> PinnedResultFuture;
}

impl<O: IsA<View>> ViewExt for O {
    fn update(&self) -> PinnedResultFuture {
        imp::view_update(self.upcast_ref())
    }
}

pub trait ViewImpl: WidgetImpl + 'static {
    fn update(&self, obj: &View) -> PinnedResultFuture {
        self.parent_update(obj)
    }
}

pub trait ViewImplExt: ObjectSubclass {
    fn parent_update(&self, obj: &View) -> PinnedResultFuture;
}

impl<T: ViewImpl> ViewImplExt for T {
    fn parent_update(&self, obj: &View) -> PinnedResultFuture {
        unsafe {
            let data = Self::type_data();
            let parent_class = &*(data.as_ref().parent_class() as *mut imp::ViewClass);
            (parent_class.update)(obj)
        }
    }
}

unsafe impl<T: ViewImpl> IsSubclassable<T> for View {
    fn class_init(class: &mut glib::Class<Self>) {
        <gtk::Widget as IsSubclassable<T>>::class_init(class);

        let klass = class.as_mut();
        klass.update = update_trampoline::<T>;
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <gtk::Widget as IsSubclassable<T>>::instance_init(instance);
    }
}

// Virtual method default implementation trampolines
fn update_trampoline<T: ObjectSubclass>(this: &View) -> PinnedResultFuture
where
    T: ViewImpl,
{
    let imp = T::from_instance(this.dynamic_cast_ref::<T::Type>().unwrap());
    imp.update(this)
}
