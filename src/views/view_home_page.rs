/* view_home_page.rs
 *
 * Copyright 2021 Visvesh Subramanian <visveshs.blogspot.com>
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
    plugins::{Plugin, Registrar},
    views::View,
    ViewExt,
};
use gtk::{
    glib::{self, object::ObjectExt, subclass::prelude::*},
    prelude::*,
};

mod imp {
    use crate::{
        plugins::{PluginObject, PluginSummaryRow, Registrar},
        views::{PinnedResultFuture, View, ViewImpl},
        Settings, ViewExt,
    };
    use adw::prelude::*;
    use gtk::{
        gio,
        glib::{self, subclass::Signal, Cast},
        {subclass::prelude::*, CompositeTemplate},
    };

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/home_page.ui")]
    pub struct ViewHomePage {
        pub settings: Settings,

        #[template_child]
        pub user_selected_data: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub all_data: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub all_data_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub size_group: TemplateChild<gtk::SizeGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewHomePage {
        const NAME: &'static str = "HealthViewHomePage";
        type ParentType = View;
        type Type = super::ViewHomePage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            unsafe {
                // FIXME: This really shouldn't be necessary.
                obj.as_ref().upcast_ref::<View>().init_template();
            }
        }
    }

    impl WidgetImpl for ViewHomePage {}

    impl ObjectImpl for ViewHomePage {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("view-changed", &[], glib::Type::UNIT.into()).build()]
            });

            SIGNALS.as_ref()
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let registrar = Registrar::instance();
            let disabled_model = registrar.disabled_plugins();
            let enabled_model = registrar.enabled_plugins();
            let disabled_filter =
                gtk::CustomFilter::new(glib::clone!(@weak obj => @default-panic, move |o| {
                    obj.filter_plugin(o.clone().downcast::<PluginObject>().unwrap().plugin(), true)
                }));
            let enabled_filter =
                gtk::CustomFilter::new(glib::clone!(@weak obj => @default-panic, move |o| {
                    obj.filter_plugin(o.clone().downcast::<PluginObject>().unwrap().plugin(), true)
                }));
            let disabled_filter_model =
                gtk::FilterListModel::new(Some(&disabled_model), Some(&disabled_filter));
            let enabled_filter_model =
                gtk::FilterListModel::new(Some(&enabled_model), Some(&enabled_filter));

            self.user_selected_data.bind_model(
                Some(&enabled_filter_model),
                glib::clone!(@strong self.size_group as sg => @default-panic, move |o| {
                    let summary = o.clone()
                        .downcast::<PluginObject>()
                        .unwrap()
                        .plugin()
                        .summary();

                        sg.add_widget(&summary);

                        summary.upcast()
                }),
            );
            self.all_data.bind_model(
                Some(&disabled_filter_model),
                glib::clone!(@strong self.size_group as sg => @default-panic, move |o| {
                    let overview = o.clone()
                        .downcast::<PluginObject>()
                        .unwrap()
                        .plugin()
                        .overview();

                        sg.add_widget(&overview);

                        overview.upcast()
                }),
            );

            self.user_selected_data.connect_row_selected(
                glib::clone!(@weak obj => move |list_box, row| {
                    if let Some(row) = row {
                        let summary = row.downcast_ref::<PluginSummaryRow>().unwrap();
                        let plugin_name = summary.plugin_name().unwrap();
                        obj.stack().set_visible_child_name(&plugin_name);
                        obj.emit_by_name::<()>("view-changed", &[]);
                        list_box.unselect_all();
                    }
                }),
            );

            if disabled_model.is_empty() {
                self.all_data_box.set_visible(false);
            }

            let stack = obj.stack();
            for plugin in enabled_model.iter() {
                plugin.update();
                stack.add_named(&plugin.details(), Some(plugin.name()));
            }
            stack.set_visible_child_name("add_data_page")
        }
    }

    impl ViewImpl for ViewHomePage {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        let registrar = Registrar::instance();
                        let enabled_model = registrar.enabled_plugins();
                        for plugin in enabled_model.iter() {
                            plugin.update();
                        }
                        send.resolve(Ok(()));
                    });
                }),
            ))
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct ViewHomePage(ObjectSubclass<imp::ViewHomePage>)
        @extends gtk::Widget, View,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewHomePage {
    pub fn back(&self) {
        self.stack().set_visible_child_name("add_data_page");
    }

    /// Connect to the `view-changed` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `view-changed` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_view_changed<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("view-changed", false, move |_| {
            callback();
            None
        })
    }

    pub fn current_page(&self) -> String {
        self.stack().visible_child_name().unwrap().to_string()
    }

    pub fn disable_current_plugin(&self) {
        let self_ = self.imp();
        let registrar = Registrar::instance();
        let current_plugin = self.current_page();

        registrar.disable_plugin(&current_plugin);
        self_.all_data_box.set_visible(true);
        self_.settings.set_enabled_plugins(
            &self_
                .settings
                .enabled_plugins()
                .iter()
                .filter(|s| *s != &current_plugin)
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
        );
    }

    /// Create a new [ViewHomePage] to display previous activities.
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewHomePage")
    }

    fn filter_plugin(&self, plugin: Box<dyn Plugin>, enabled: bool) -> bool {
        let registrar = Registrar::instance();
        if enabled {
            registrar.enabled_plugins().contains(plugin.name())
        } else {
            registrar.disabled_plugins().contains(plugin.name())
        }
    }

    fn imp(&self) -> &imp::ViewHomePage {
        imp::ViewHomePage::from_instance(self)
    }
}
