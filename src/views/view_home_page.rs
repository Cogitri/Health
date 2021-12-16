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

use crate::{plugins::Registrar, views::View, ViewExt};
use gtk::{
    glib::{self, object::ObjectExt, subclass::prelude::*},
    prelude::*,
};

mod imp {
    use crate::{
        plugins::{PluginObject, PluginOverviewRow, PluginSummaryRow, Registrar},
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
        #[template_child]
        pub enabled_plugins_stack: TemplateChild<gtk::Stack>,
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

            let sorter = gtk::CustomSorter::new(|a, b| {
                a.downcast_ref::<PluginObject>()
                    .unwrap()
                    .plugin()
                    .name()
                    .cmp(b.downcast_ref::<PluginObject>().unwrap().plugin().name())
                    .into()
            });
            let enabled_model_sorted = gtk::SortListModel::new(Some(&enabled_model), Some(&sorter));
            let disabled_model_sorted =
                gtk::SortListModel::new(Some(&disabled_model), Some(&sorter));

            self.user_selected_data.bind_model(
                Some(&enabled_model_sorted),
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
                Some(&disabled_model_sorted),
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
            self.all_data
                .connect_row_selected(glib::clone!(@weak obj => move |list_box, row| {
                    if let Some(row) = row {
                        let overview = row.downcast_ref::<PluginOverviewRow>().unwrap();
                        let plugin_name = overview.plugin_name().unwrap();
                        obj.stack().set_visible_child_name(&plugin_name);
                        obj.emit_by_name::<()>("view-changed", &[]);
                        list_box.unselect_all();
                    }
                }));

            if disabled_model.is_empty() {
                self.all_data_box.set_visible(false);
            }

            if enabled_model.is_empty() {
                self.enabled_plugins_stack
                    .set_visible_child_name("no-plugins-enabled");
            }

            let stack = obj.stack();
            for plugin in enabled_model.iter() {
                plugin.update();
                stack.add_named(&plugin.details(), Some(plugin.name()));
            }

            for plugin in disabled_model.iter() {
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

    pub fn is_current_plugin_enabled(&self) -> bool {
        let registrar = Registrar::instance();
        let current_page = self.current_page();
        registrar
            .enabled_plugins()
            .iter()
            .any(|p| p.name() == current_page)
    }

    pub fn disable_current_plugin(&self) {
        let self_ = self.imp();
        let registrar = Registrar::instance();
        let current_plugin = self.current_page();

        registrar.disable_plugin(&current_plugin);
        self_.all_data_box.set_visible(true);
        self_.settings.set_enabled_plugins(
            self_
                .settings
                .enabled_plugins()
                .iter()
                .filter(|s| *s != &current_plugin)
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
        );
        if registrar.enabled_plugins().is_empty() {
            self_
                .enabled_plugins_stack
                .set_visible_child_name("no-plugins-enabled")
        }
    }

    pub fn enable_current_plugin(&self) {
        let self_ = self.imp();
        let registrar = Registrar::instance();
        let current_plugin = self.current_page();

        let mut enabled_plugins = self_.settings.enabled_plugins();
        enabled_plugins.push(current_plugin.clone());
        registrar.enable_plugin(&current_plugin);
        self_.settings.set_enabled_plugins(
            enabled_plugins
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
        );
        self_
            .enabled_plugins_stack
            .set_visible_child_name("plugin-list");
        if registrar.disabled_plugins().is_empty() {
            self_.all_data_box.set_visible(false);
        }
    }

    /// Create a new [ViewHomePage] to display previous activities.
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ViewHomePage")
    }

    fn imp(&self) -> &imp::ViewHomePage {
        imp::ViewHomePage::from_instance(self)
    }
}
