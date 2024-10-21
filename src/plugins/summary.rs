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
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::{self, clone};
use std::str::FromStr;

mod imp {
    use crate::{plugins::PluginName, prelude::*};
    use adw::subclass::prelude::*;
    use gtk::{gio, glib, prelude::*};
    use once_cell::unsync::OnceCell;
    use std::str::FromStr;

    #[repr(C)]
    pub struct PluginSummaryRowClass {
        pub parent_class: adw::ffi::AdwActionRowClass,
        pub update: fn(&super::PluginSummaryRow) -> PinnedResultFuture<()>,
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
        PluginSummaryRow::from_obj(this).update(this)
    }

    pub(super) fn plugin_summary_row_update(
        this: &super::PluginSummaryRow,
    ) -> PinnedResultFuture<()> {
        let klass = this.class();

        (klass.as_ref().update)(this)
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
            klass.update = update_default_trampoline;
        }
    }

    impl ObjectImpl for PluginSummaryRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecString::builder("plugin-name")
                    .construct_only()
                    .readwrite()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "plugin-name" => self.plugin_name.get().unwrap().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "plugin-name" => self
                    .plugin_name
                    .set(PluginName::from_str(value.get::<&str>().unwrap()).unwrap())
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
    pub fn new(plugin_name: PluginName) -> Self {
        let obj: Self = glib::Object::builder()
            .property("plugin-name", &plugin_name)
            .build();
        obj.bind_right_click();

        obj
    }
}

/// [PluginSummaryRowExt] is implemented by all subclasses of [PluginSummaryRow].
pub trait PluginSummaryRowExt {
    /// Update the [PluginSummaryRow]'s data
    fn update(&self) -> PinnedResultFuture<()>;

    fn plugin_name(&self) -> PluginName;

    fn bind_right_click(&self);
}

impl<O: IsA<PluginSummaryRow>> PluginSummaryRowExt for O {
    fn update(&self) -> PinnedResultFuture<()> {
        imp::plugin_summary_row_update(self.upcast_ref())
    }

    fn plugin_name(&self) -> PluginName {
        PluginName::from_str(&self.property::<String>("plugin-name")).unwrap()
    }

    fn bind_right_click(&self) {
        let obj = self.upcast_ref();
        let on_rightclick = clone!(
            #[weak]
            obj,
            move |(x, y)| {
                let menu = gio::Menu::new();
                let plugin_name = obj.property::<String>("plugin-name");
                menu.append(
                    Some("Disable Plugin"),
                    Some(&format!("win.disable-plugin::{plugin_name}")),
                );

                let popover = gtk::PopoverMenu::builder()
                    .menu_model(&menu)
                    .pointing_to(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1))
                    .build();
                obj.connect_destroy(clone!(
                    #[weak]
                    popover,
                    move |_| popover.unparent()
                ));
                popover.set_parent(&obj);
                popover.popup();
            }
        );
        let on_long_press = on_rightclick.clone();

        let long_press = gtk::GestureLongPress::new();
        long_press.connect_pressed(move |_, x, y| {
            on_long_press((x, y));
        });

        let right_click = gtk::GestureClick::builder()
            .button(gtk::gdk::BUTTON_SECONDARY)
            .build();
        right_click.connect_pressed(move |_, _, x, y| {
            on_rightclick((x, y));
        });
        obj.add_controller(long_press);
        obj.add_controller(right_click);
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
            let parent_class = &*(data.as_ref().parent_class() as *mut imp::PluginSummaryRowClass);
            (parent_class.update)(obj)
        }
    }
}

unsafe impl<T: PluginSummaryRowImpl> IsSubclassable<T> for PluginSummaryRow {
    fn class_init(class: &mut glib::Class<Self>) {
        <adw::ActionRow as IsSubclassable<T>>::class_init(class.upcast_ref_mut());

        let klass = class.as_mut();
        klass.update = update_trampoline::<T>;
    }

    fn instance_init(instance: &mut glib::subclass::InitializingObject<T>) {
        <adw::ActionRow as IsSubclassable<T>>::instance_init(instance);
    }
}

// Virtual method default implementation trampolines
fn update_trampoline<T>(this: &PluginSummaryRow) -> PinnedResultFuture<()>
where
    T: PluginSummaryRowImpl + ObjectSubclass,
{
    let imp = T::from_obj(this.dynamic_cast_ref::<T::Type>().unwrap());
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
    use super::{PluginName, PluginSummaryRow};
    use crate::utils::init_gtk;

    #[gtk::test]
    fn new() {
        init_gtk();
        PluginSummaryRow::new(PluginName::Activities);
    }
}
