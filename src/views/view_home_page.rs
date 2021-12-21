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
    plugins::{PluginDetailsExt, PluginObject, PluginSummaryRow, PluginSummaryRowExt, Registrar},
    views::{View, ViewExt},
};
use gtk::{
    glib::{self, object::ObjectExt, subclass::prelude::*},
    prelude::*,
};

mod imp {
    use crate::{
        core::Settings,
        plugins::{PluginObject, PluginOverviewRow, PluginSummaryRow, Registrar},
        views::{PinnedResultFuture, View, ViewExt, ViewImpl},
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
            Self::bind_template_callbacks(klass);
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
                glib::clone!(@weak obj => @default-panic, move |o| {
                    obj.handle_user_selected_data_bind_model(o)
                }),
            );
            self.all_data.bind_model(
                Some(&disabled_model_sorted),
                glib::clone!(@weak obj => @default-panic, move |o| {
                    obj.handle_all_data_bind_model(o)
                }),
            );

            if disabled_model.is_empty() {
                self.all_data_box.set_visible(false);
            }

            if enabled_model.is_empty() {
                self.enabled_plugins_stack
                    .set_visible_child_name("no-plugins-enabled");
            }

            obj.stack().set_visible_child_name("add_data_page")
        }
    }

    impl ViewImpl for ViewHomePage {
        fn update(&self, obj: &View) -> PinnedResultFuture {
            Box::pin(gio::GioFuture::new(
                obj,
                glib::clone!(@weak obj => move |_, _, send| {
                    gtk_macros::spawn!(async move {
                        obj.downcast_ref::<Self::Type>().unwrap().update().await;
                        send.resolve(Ok(()));
                    });
                }),
            ))
        }
    }

    #[gtk::template_callbacks(subclass)]
    impl ViewHomePage {
        #[template_callback]
        fn handle_user_selected_data_row_selected(
            &self,
            row: Option<gtk::ListBoxRow>,
            list_box: gtk::ListBox,
        ) {
            if let Some(row) = row {
                let summary = row.downcast_ref::<PluginSummaryRow>().unwrap();
                let plugin_name = summary.plugin_name().unwrap();
                self.instance()
                    .open_plugin_details(&list_box, &plugin_name, true);
            }
        }

        #[template_callback]
        fn handle_all_data_row_selected(
            &self,
            row: Option<gtk::ListBoxRow>,
            list_box: gtk::ListBox,
        ) {
            if let Some(row) = row {
                let overview = row.downcast_ref::<PluginOverviewRow>().unwrap();
                let plugin_name = overview.plugin_name().unwrap();
                self.instance()
                    .open_plugin_details(&list_box, &plugin_name, false);
            }
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
        let stack = self.stack();
        stack.remove(&stack.child_by_name(&self.current_page()).unwrap());
        stack.set_visible_child_name("add_data_page");
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

    fn handle_user_selected_data_bind_model(&self, object: &glib::Object) -> gtk::Widget {
        let summary = object
            .downcast_ref::<PluginObject>()
            .unwrap()
            .plugin()
            .summary();

        self.imp().size_group.add_widget(&summary);

        summary.upcast()
    }

    fn handle_all_data_bind_model(&self, object: &glib::Object) -> gtk::Widget {
        let overview = object
            .downcast_ref::<PluginObject>()
            .unwrap()
            .plugin()
            .overview();

        self.imp().size_group.add_widget(&overview);

        overview.upcast()
    }

    fn open_plugin_details(&self, list_box: &gtk::ListBox, plugin_name: &str, enabled: bool) {
        let registrar = Registrar::instance();
        let stack = self.stack();
        let plugin = if enabled {
            registrar.enabled_plugin_by_name(plugin_name).unwrap()
        } else {
            registrar.disabled_plugin_by_name(plugin_name).unwrap()
        };
        let details = plugin.details(enabled);

        stack.add_named(&details, Some(plugin_name));
        stack.set_visible_child_name(plugin_name);
        self.emit_by_name::<()>("view-changed", &[]);
        list_box.unselect_all();

        gtk_macros::spawn!(async move {
            if let Err(e) = details.update().await {
                glib::g_warning!(
                    crate::config::LOG_DOMAIN,
                    "Couldn't update plugin's details: {}",
                    e
                );
            }
        });
    }

    fn imp(&self) -> &imp::ViewHomePage {
        imp::ViewHomePage::from_instance(self)
    }

    async fn update(&self) {
        let self_ = self.imp();
        let mut i = 0;
        while let Some(row) = self_.user_selected_data.row_at_index(i) {
            if let Err(e) = row
                .downcast_ref::<PluginSummaryRow>()
                .unwrap()
                .update()
                .await
            {
                glib::g_warning!(crate::config::LOG_DOMAIN, "Couldn't update plugin: {}", e);
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::ViewHomePage;
    use crate::utils::init_gtk;

    #[test]
    fn new() {
        init_gtk();
        ViewHomePage::new();
    }
}
