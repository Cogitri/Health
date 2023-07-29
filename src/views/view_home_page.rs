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
    model::User,
    plugins::{PluginDetails, PluginName, PluginObject, PluginSummaryRow, Registrar},
    prelude::*,
};
use adw::prelude::*;
use gtk::glib::{self, subclass::prelude::*};
use std::str::FromStr;

mod imp {
    use crate::{
        core::{Database, Settings},
        plugins::{PluginObject, PluginOverviewRow, PluginSummaryRow, Registrar},
        prelude::*,
    };
    use adw::{prelude::*, subclass::prelude::*};
    use gtk::{
        glib::{self, subclass::Signal, Cast},
        CompositeTemplate,
    };
    use num_traits::cast::ToPrimitive;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/view_home_page.ui")]
    pub struct ViewHomePage {
        pub settings: Settings,
        pub database: Database,

        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub user_selected_data: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub all_data: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub all_data_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub size_group: TemplateChild<gtk::SizeGroup>,
        #[template_child]
        pub summary_size_group: TemplateChild<gtk::SizeGroup>,
        #[template_child]
        pub enabled_plugins_stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ViewHomePage {
        const NAME: &'static str = "HealthViewHomePage";
        type ParentType = adw::Bin;
        type Type = super::ViewHomePage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl WidgetImpl for ViewHomePage {}

    impl ObjectImpl for ViewHomePage {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("view-changed")
                    .param_types([String::static_type()])
                    .build()]
            });

            SIGNALS.as_ref()
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            let registrar = Registrar::instance();
            let disabled_model = registrar.disabled_plugins();
            let enabled_model = registrar.enabled_plugins();

            let sorter = gtk::CustomSorter::new(|a, b| {
                a.downcast_ref::<PluginObject>()
                    .unwrap()
                    .plugin()
                    .name()
                    .to_u8()
                    .unwrap()
                    .cmp(
                        &b.downcast_ref::<PluginObject>()
                            .unwrap()
                            .plugin()
                            .name()
                            .to_u8()
                            .unwrap(),
                    )
                    .into()
            });
            let enabled_model_sorted =
                gtk::SortListModel::new(Some(enabled_model), Some(sorter.clone()));
            let disabled_model_sorted = gtk::SortListModel::new(Some(disabled_model), Some(sorter));

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

            obj.handle_registrar_plugins_changed();
            registrar.connect_plugins_updated(glib::clone!(@weak obj => move |_| {
                obj.handle_registrar_plugins_changed();
            }));
        }
    }

    impl BinImpl for ViewHomePage {}

    #[gtk::template_callbacks]
    impl ViewHomePage {
        #[template_callback]
        fn handle_user_selected_data_row_activated(
            &self,
            row: gtk::ListBoxRow,
            list_box: gtk::ListBox,
        ) {
            let summary = row.downcast_ref::<PluginSummaryRow>().unwrap();
            let plugin_name = summary.plugin_name();
            self.obj().open_plugin_details(list_box, plugin_name, true);
        }

        #[template_callback]
        fn handle_all_data_row_activated(&self, row: gtk::ListBoxRow, list_box: gtk::ListBox) {
            let overview = row.downcast_ref::<PluginOverviewRow>().unwrap();
            let plugin_name = overview.plugin_name();
            self.obj().open_plugin_details(list_box, plugin_name, false);
        }
    }
}

glib::wrapper! {
    /// An implementation of [View] visualizes activities the user recently did.
    pub struct ViewHomePage(ObjectSubclass<imp::ViewHomePage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ViewHomePage {
    pub fn back(&self) {
        let imp = self.imp();
        imp.stack
            .remove(&imp.stack.child_by_name(&self.current_page()).unwrap());
        imp.stack.set_visible_child_name("home");
    }

    /// Connect to the `view-changed` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `view-changed` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_view_changed<F: Fn(&Self, PluginName) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("view-changed", false, move |values| {
            callback(
                &values[0].get().unwrap(),
                PluginName::from_str(values[1].get::<&str>().unwrap()).unwrap(),
            );
            None
        })
    }

    pub fn current_page(&self) -> String {
        self.imp().stack.visible_child_name().unwrap().to_string()
    }

    pub async fn get_user(&self) -> User {
        let imp = self.imp();
        let user_id = i64::from(imp.settings.active_user_id());
        let user = &imp.database.user(user_id).await.unwrap();
        user.clone()
    }

    pub fn is_current_plugin_enabled(&self) -> bool {
        let registrar = Registrar::instance();
        let current_page = self.current_page();
        registrar
            .enabled_plugins()
            .iter()
            .any(|p| p.name().as_ref() == current_page.as_str())
    }

    pub async fn disable_current_plugin(&self) {
        let imp = self.imp();
        let registrar = Registrar::instance();
        let user = self.get_user().await;

        if let Ok(current_plugin) = PluginName::from_str(&self.current_page()) {
            registrar.disable_plugin(current_plugin);
            imp.all_data_box.set_visible(true);
            user.set_enabled_plugins(Some(
                user.enabled_plugins()
                    .unwrap()
                    .drain(..)
                    .filter(|s| *s != current_plugin)
                    .collect::<Vec<PluginName>>()
                    .as_slice()
                    .to_vec(),
            ));
            if registrar.enabled_plugins().is_empty() {
                imp.enabled_plugins_stack
                    .set_visible_child_name("no-plugins-enabled")
            }
        }
    }

    pub async fn enable_current_plugin(&self) {
        let imp = self.imp();
        let registrar = Registrar::instance();
        let user = self.get_user().await;

        if let Ok(current_plugin) = PluginName::from_str(&self.current_page()) {
            let mut enabled_plugins = user.enabled_plugins().unwrap();
            enabled_plugins.push(current_plugin);
            registrar.enable_plugin(current_plugin);
            user.set_enabled_plugins(Some(enabled_plugins));
            imp.enabled_plugins_stack
                .set_visible_child_name("plugin-list");
            if registrar.disabled_plugins().is_empty() {
                imp.all_data_box.set_visible(false);
            }
        }
    }

    /// Create a new [ViewHomePage] to display previous activities.
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn handle_user_selected_data_bind_model(&self, object: &glib::Object) -> gtk::Widget {
        let summary = object
            .downcast_ref::<PluginObject>()
            .unwrap()
            .plugin()
            .summary();

        gtk_macros::spawn!(glib::clone!(@weak summary => async move {
            if let Err(e) = summary.update().await {
                glib::g_warning!(crate::config::LOG_DOMAIN, "Couldn't update plugin: {e}");
            }
        }));

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
        self.imp()
            .summary_size_group
            .add_widget(&overview.icon_widget());

        overview.upcast()
    }

    fn open_plugin_details(&self, list_box: gtk::ListBox, plugin_name: PluginName, enabled: bool) {
        let imp = self.imp();
        let registrar = Registrar::instance();
        let plugin = if enabled {
            registrar.enabled_plugin_by_name(plugin_name).unwrap()
        } else {
            registrar.disabled_plugin_by_name(plugin_name).unwrap()
        };
        let details = plugin.details(!enabled);

        imp.stack.add_named(&details, Some(plugin_name.as_ref()));

        gtk_macros::spawn!(glib::clone!(@weak self as obj => async move {
            obj.update_view(details, plugin_name, list_box).await;
        }));
    }

    pub async fn update(&self) {
        let imp = self.imp();
        let mut i = 0;
        while let Some(row) = imp.user_selected_data.row_at_index(i) {
            if let Err(e) = row
                .downcast_ref::<PluginSummaryRow>()
                .unwrap()
                .update()
                .await
            {
                glib::g_warning!(crate::config::LOG_DOMAIN, "Couldn't update plugin: {e}");
            }
            i += 1;
        }
    }

    async fn update_view(
        &self,
        details: PluginDetails,
        plugin_name: PluginName,
        list_box: gtk::ListBox,
    ) {
        let imp = self.imp();

        if let Err(e) = details.update().await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Couldn't update plugin's details: {e}",
            );
        }

        imp.stack.set_visible_child_name(plugin_name.as_ref());
        self.emit_by_name::<()>("view-changed", &[&plugin_name.as_ref()]);
        list_box.unselect_all();
    }

    fn handle_registrar_plugins_changed(&self) {
        let imp = self.imp();
        let registrar = Registrar::instance();
        let disabled_model = registrar.disabled_plugins();
        let enabled_model = registrar.enabled_plugins();

        imp.all_data_box.set_visible(!disabled_model.is_empty());

        if enabled_model.is_empty() {
            imp.enabled_plugins_stack
                .set_visible_child_name("no-plugins-enabled");
        } else {
            imp.enabled_plugins_stack
                .set_visible_child_name("plugin-list")
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
