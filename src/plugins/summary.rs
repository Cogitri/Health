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

use crate::{
    plugins::{
        PluginActivitiesSummaryRow, PluginCaloriesSummaryRow, PluginName, PluginStepsSummaryRow,
        PluginWeightSummaryRow,
    },
    prelude::*,
};
use adw::subclass::prelude::*;
use gtk::glib::{self, prelude::*};
use std::str::FromStr;

mod imp {
    use crate::{plugins::PluginName, prelude::*};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*, subclass::prelude::*};
    use once_cell::unsync::OnceCell;
    use std::str::FromStr;

    #[repr(C)]
    pub struct PluginSummaryRowClass {
        pub parent_class: adw::ffi::AdwActionRowClass,
        pub update: Option<unsafe fn(&super::PluginSummaryRow) -> PinnedResultFuture<()>>,
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
        pub plugin_name: OnceCell<PluginName>,
    }

    // Virtual method default implementation trampolines
    fn update_default_trampoline(this: &super::PluginSummaryRow) -> PinnedResultFuture<()> {
        PluginSummaryRow::from_instance(this).update(this)
    }

    pub(super) unsafe fn plugin_summary_row_update(
        this: &super::PluginSummaryRow,
    ) -> PinnedResultFuture<()> {
        let klass = &*(this.class() as *const _ as *const PluginSummaryRowClass);

        (klass.update.unwrap())(this)
    }

    impl PluginSummaryRow {
        fn update(&self, obj: &super::PluginSummaryRow) -> PinnedResultFuture<()> {
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
                "plugin-name" => self
                    .plugin_name
                    .set(PluginName::from_str(&value.get::<String>().unwrap()).unwrap())
                    .unwrap(),
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
    /// A [PluginSummaryRow] displays a quick glance of info for the user (e.g. "X/Y steps done today").
    ///
    /// See [PluginSummaryExt] for what methods this exposes.
    pub struct PluginSummaryRow(ObjectSubclass<imp::PluginSummaryRow>)
    @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PluginSummaryRow {
    /// Create a new [PluginSummaryRow]
    pub fn new(plugin_name: &str) -> Self {
        glib::Object::new(&[("plugin-name", &plugin_name)])
            .expect("Failed to create PluginSummaryRow")
    }

    pub fn plugin_name(&self) -> PluginName {
        PluginName::from_str(&self.property::<String>("plugin-name")).unwrap()
    }
}

/// [PluginSummaryRowExt] is implemented by all subclasses of [PluginSummaryRow].
pub trait PluginSummaryRowExt {
    /// Update the [PluginSummaryRow]'s data
    fn update(&self) -> PinnedResultFuture<()>;
}

impl<O: IsA<PluginSummaryRow>> PluginSummaryRowExt for O {
    fn update(&self) -> PinnedResultFuture<()> {
        unsafe { imp::plugin_summary_row_update(self.upcast_ref()) }
    }
}

pub trait PluginSummaryRowImpl: ActionRowImpl + 'static {
    fn update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
        self.parent_update(obj)
    }
}

pub trait PluginSummaryRowImplExt: ObjectSubclass {
    fn parent_update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()>;
}

impl<T: PluginSummaryRowImpl> PluginSummaryRowImplExt for T {
    fn parent_update(&self, obj: &PluginSummaryRow) -> PinnedResultFuture<()> {
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
unsafe fn update_trampoline<T: ObjectSubclass>(this: &PluginSummaryRow) -> PinnedResultFuture<()>
where
    T: PluginSummaryRowImpl,
{
    let instance = &*(this as *const _ as *const T::Instance);
    let imp = instance.impl_();
    imp.update(this)
}

impl From<PluginName> for PluginSummaryRow {
    fn from(plugin_name: PluginName) -> Self {
        match plugin_name {
            PluginName::Activities => PluginActivitiesSummaryRow::new(plugin_name).upcast(),
            PluginName::Calories => PluginCaloriesSummaryRow::new(plugin_name).upcast(),
            PluginName::Steps => PluginStepsSummaryRow::new(plugin_name).upcast(),
            PluginName::Weight => PluginWeightSummaryRow::new(plugin_name).upcast(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::PluginSummaryRow;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        PluginSummaryRow::new("");
    }
}
