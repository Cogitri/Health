/* PluginSummaryRow.rs
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

use super::{
    PluginActivitiesSummaryRow, PluginCaloriesSummaryRow, PluginStepsSummaryRow,
    PluginWeightSummaryRow,
};
use crate::properties_setter_getter;
use adw::subclass::prelude::*;
use anyhow::Result;
use gtk::glib::{self, prelude::*};
use std::{future::Future, pin::Pin};

pub type PinnedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + 'static>>;

mod imp {
    use super::PinnedResultFuture;
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;

    pub type PluginSummaryRowInstance = super::PluginSummaryRow;

    #[repr(C)]
    pub struct PluginSummaryRowClass {
        pub parent_class: adw::ffi::AdwActionRowClass,
        pub update: Option<unsafe fn(&PluginSummaryRowInstance) -> PinnedResultFuture>,
    }

    unsafe impl ClassStruct for PluginSummaryRowClass {
        type Type = PluginSummaryRow;
    }

    impl std::ops::Deref for PluginSummaryRowClass {
        type Target = glib::Class<glib::Object>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*(self as *const Self).cast::<Self::Target>() }
        }
    }

    impl std::ops::DerefMut for PluginSummaryRowClass {
        fn deref_mut(&mut self) -> &mut glib::Class<glib::Object> {
            unsafe { &mut *(self as *mut Self).cast::<glib::Class<glib::Object>>() }
        }
    }

    #[derive(Debug, Default)]
    pub struct PluginSummaryRow {
        pub plugin_name: OnceCell<String>,
    }

    // Virtual method default implementation trampolines
    fn update_default_trampoline(this: &PluginSummaryRowInstance) -> PinnedResultFuture {
        PluginSummaryRow::from_instance(this).update(this)
    }

    pub(super) unsafe fn plugin_summary_row_update(
        this: &PluginSummaryRowInstance,
    ) -> PinnedResultFuture {
        let klass = &*(this.class() as *const _ as *const PluginSummaryRowClass);

        (klass.update.unwrap())(this)
    }

    impl PluginSummaryRow {
        fn update(&self, obj: &super::PluginSummaryRow) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(obj, move |_, _, send| {
                send.resolve(Ok(()));
            }))
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PluginSummaryRow {
        const NAME: &'static str = "HealthPluginSummaryRow";
        type ParentType = adw::ActionRow;
        type Type = super::PluginSummaryRow;
        type Class = PluginSummaryRowClass;

        fn class_init(klass: &mut Self::Class) {
            klass.update = Some(update_default_trampoline);
        }
    }

    impl ObjectImpl for PluginSummaryRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::new(
                    "plugin-name",
                    "plugin-name",
                    "plugin-name",
                    None,
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "plugin-name" => self.plugin_name.get().unwrap().to_value(),
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
                "plugin-name" => self.plugin_name.set(value.get().unwrap()).unwrap(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for PluginSummaryRow {}
    impl ListBoxRowImpl for PluginSummaryRow {}
    impl PreferencesRowImpl for PluginSummaryRow {}
    impl ActionRowImpl for PluginSummaryRow {}
}

glib::wrapper! {
    /// [PluginSummaryRow] is a toplevel container that is implemented by all other PluginSummaryRows of Health.
    pub struct PluginSummaryRow(ObjectSubclass<imp::PluginSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginSummaryRow {
    pub fn new(plugin_name: &str) -> Self {
        glib::Object::new(&[("plugin-name", &plugin_name)])
            .expect("Failed to create PluginSummaryRow")
    }

    properties_setter_getter!("plugin-name", String);
}

pub trait PluginSummaryRowExt {
    fn update(&self) -> PinnedResultFuture;
}

impl<O: IsA<PluginSummaryRow>> PluginSummaryRowExt for O {
    fn update(&self) -> PinnedResultFuture {
        unsafe { imp::plugin_summary_row_update(self.upcast_ref()) }
    }
}

pub trait PluginSummaryRowImpl: ActionRowImpl + 'static {
    fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture {
        self.parent_update(obj)
    }
}

pub trait PluginSummaryRowImplExt: ObjectSubclass {
    fn parent_update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture;
}

impl<T: PluginSummaryRowImpl> PluginSummaryRowImplExt for T {
    fn parent_update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture {
        unsafe {
            let data = Self::type_data();
            let parent_class = data
                .as_ref()
                .parent_class()
                .cast::<imp::PluginSummaryRowClass>();
            if let Some(ref f) = (*parent_class).update {
                f(obj)
            } else {
                unimplemented!()
            }
        }
    }
}

unsafe impl<T: PluginSummaryRowImpl> IsSubclassable<T> for PluginSummaryRow {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::ActionRow as IsSubclassable<T>>::class_init(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.update = Some(update_trampoline::<T>);
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::ActionRow as IsSubclassable<T>>::instance_init(instance);
    }
}

// Virtual method default implementation trampolines
unsafe fn update_trampoline<T: ObjectSubclass>(this: &PluginSummaryRow) -> PinnedResultFuture
where
    T: PluginSummaryRowImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.update(this)
}

impl From<&str> for PluginSummaryRow {
    fn from(plugin_name: &str) -> Self {
        match plugin_name {
            "activities" => PluginActivitiesSummaryRow::new(plugin_name).upcast(),
            "calories" => PluginCaloriesSummaryRow::new(plugin_name).upcast(),
            "weight" => PluginWeightSummaryRow::new(plugin_name).upcast(),
            "steps" => PluginStepsSummaryRow::new(plugin_name).upcast(),
            _ => unimplemented!(),
        }
    }
}
