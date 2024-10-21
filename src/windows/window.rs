/* window.rs
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

use crate::plugins::details::PluginDetailsExt;
use crate::views::ViewAddActivity;
use crate::views::ViewAddWeight;
use crate::{
    core::{i18n_f, Database},
    model::User,
    plugins::{PluginName, Registrar},
    sync::{google_fit::GoogleFitSyncProvider, new_db_receiver, sync_provider::SyncProvider},
};
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*},
    prelude::*,
};
use gtk_macros::action;

mod imp {
    use crate::{core::Settings, views::ViewHomePage};
    use gtk::{
        glib::{self, SourceId},
        prelude::*,
        subclass::prelude::*,
        CompositeTemplate,
    };
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct WindowMut {
        pub current_height: i32,
        pub current_width: i32,
        pub sync_source_id: Option<SourceId>,
    }

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/dev/Cogitri/Health/ui/window.ui")]
    pub struct Window {
        pub inner: RefCell<WindowMut>,
        pub settings: Settings,

        #[template_child]
        pub error_infobar: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub view_home_page: TemplateChild<ViewHomePage>,
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "HealthWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::Window;

        fn class_init(klass: &mut Self::Class) {
            ViewHomePage::static_type();
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.setup();
            obj.setup_actions();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    /// The toplevel application window that holds all other widgets.
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl Window {
    /// Create a new [Window].
    ///
    /// # Arguments
    /// * `app` - The application to use.
    pub fn new<P: IsA<gtk::Application>>(app: &P) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn go_to_add_weight(&self) {
        let page = ViewAddWeight::new();
        self.imp().navigation_view.push(&page);
    }

    fn go_to_add_activity(&self) {
        let page = ViewAddActivity::new();
        self.imp().navigation_view.push(&page);
    }

    fn setup_actions(&self) {
        action!(
            self,
            "fullscreen",
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, _| obj.handle_fullscreen()
            )
        );
        action!(
            self,
            "go-to-add-activity",
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, _| obj.go_to_add_activity()
            )
        );
        action!(
            self,
            "go-to-add-weight",
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, _| obj.go_to_add_weight()
            )
        );
        action!(
            self,
            "disable-plugin",
            Some(glib::VariantTy::STRING),
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, arg| if let Some(plugin_name) =
                    arg.map(PluginName::try_from).and_then(Result::ok)
                {
                    obj.handle_disable_plugin(plugin_name)
                } else {
                    glib::g_warning!(crate::config::LOG_DOMAIN, "NO PLUGIN FOR NAME {arg:#?}");
                }
            )
        );
        action!(
            self,
            "enable-plugin",
            Some(glib::VariantTy::STRING),
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |_, arg| if let Some(plugin_name) =
                    arg.map(PluginName::try_from).and_then(Result::ok)
                {
                    obj.handle_enable_plugin(plugin_name)
                }
            )
        );
    }

    #[template_callback]
    fn handle_close_request(&self) -> bool {
        let imp = self.imp();
        let mut inner = imp.inner.borrow_mut();

        imp.settings.set_window_is_maximized(self.is_maximized());
        imp.settings.set_window_height(inner.current_height);
        imp.settings.set_window_width(inner.current_width);

        if let Some(source_id) = inner.sync_source_id.take() {
            source_id.remove();
        }

        if let Some(app) = self
            .application()
            .and_then(|a| a.downcast::<crate::core::Application>().ok())
        {
            app.handle_shutdown(false);
        }

        false
    }

    #[template_callback]
    fn handle_error_infobar_close(&self) {
        self.imp().error_infobar.set_reveal_child(false);
    }

    fn handle_fullscreen(&self) {
        if self.is_fullscreen() {
            self.unfullscreen();
        } else {
            self.fullscreen();
        }
    }

    #[template_callback]
    fn handle_property_default_height_notify(&self) {
        self.imp().inner.borrow_mut().current_height = self.default_height();
    }

    #[template_callback]
    fn handle_property_default_width_notify(&self) {
        self.imp().inner.borrow_mut().current_height = self.default_height();
    }

    fn setup(&self) {
        self.update();
        self.sync_data();

        // FIXME: Allow setting custom sync interval
        self.imp().inner.borrow_mut().sync_source_id = Some(glib::timeout_add_seconds_local(
            60 * 5,
            clone!(
                #[weak(rename_to = obj)]
                self,
                #[upgrade_or_panic]
                move || {
                    obj.sync_data();

                    glib::ControlFlow::Continue
                }
            ),
        ));
    }

    fn handle_sync_data_error_received(&self, err_opt: Option<anyhow::Error>) -> glib::ControlFlow {
        if let Some(e) = err_opt {
            self.show_error(&i18n_f(
                "Couldn’t sync Google Fit data due to error: {}",
                &[&e.to_string()],
            ));
        }

        glib::ControlFlow::Break
    }

    /// Display an error in a non-intrusive way.
    fn show_error(&self, err_msg: &str) {
        let imp = self.imp();

        glib::g_warning!(crate::config::LOG_DOMAIN, "{err_msg}");
        imp.error_label.set_text(err_msg);
        imp.error_infobar.set_reveal_child(true);
    }

    fn sync_data(&self) {
        let imp = self.imp();

        if imp.settings.sync_provider_setup_google_fit() {
            let (sender, receiver) = async_channel::unbounded();
            let db_sender = new_db_receiver();

            glib::spawn_future_local(clone!(
                #[weak(rename_to = obj)]
                self,
                #[upgrade_or_panic]
                async move {
                    while let Ok(v) = receiver.recv().await {
                        if obj.handle_sync_data_error_received(v) == glib::ControlFlow::Break {
                            break;
                        }
                    }
                }
            ));

            std::thread::spawn(move || {
                let mut sync_proxy = GoogleFitSyncProvider::new(db_sender);
                if let Err(e) = sync_proxy.sync_data() {
                    sender.send_blocking(Some(e)).unwrap();
                } else {
                    sender.send_blocking(None).unwrap();
                }
            });
        }
    }

    pub fn update(&self) {
        let view = self.imp().view_home_page.get();
        gtk_macros::spawn!(clone!(
            #[weak]
            view,
            async move {
                view.update().await;
            }
        ));
    }

    pub async fn open_plugin(&self, plugin_name: PluginName, enabled: bool) {
        let registrar = Registrar::instance();
        let plugin = if enabled {
            registrar.enabled_plugin_by_name(plugin_name).unwrap()
        } else {
            registrar.disabled_plugin_by_name(plugin_name).unwrap()
        };
        let details = plugin.details(!enabled);

        if let Err(e) = details.update().await {
            glib::g_warning!(
                crate::config::LOG_DOMAIN,
                "Couldn't update plugin's details: {e}",
            );
        }

        self.imp().navigation_view.push(&details);
    }

    fn handle_disable_plugin(&self, plugin_name: PluginName) {
        gtk_macros::spawn!(clone!(
            #[weak(rename_to = obj)]
            self,
            async move {
                obj.disable_plugin(plugin_name).await;
            }
        ));
    }

    fn handle_enable_plugin(&self, plugin_name: PluginName) {
        gtk_macros::spawn!(clone!(
            #[weak(rename_to = obj)]
            self,
            async move {
                obj.enable_plugin(plugin_name).await;
            }
        ));
    }

    pub async fn disable_plugin(&self, plugin_name: PluginName) {
        let database = Database::instance();
        let registrar = Registrar::instance();
        let user = self.get_user(&database).await;

        registrar.disable_plugin(plugin_name);
        user.disable_plugin(plugin_name);
        if let Err(err_msg) = database.update_user(user).await {
            glib::g_warning!(crate::config::LOG_DOMAIN, "{err_msg}");
        }
    }

    pub async fn enable_plugin(&self, plugin_name: PluginName) {
        let database = Database::instance();
        let registrar = Registrar::instance();
        let user = self.get_user(&database).await;

        registrar.enable_plugin(plugin_name);
        user.enable_plugin(plugin_name);
        if let Err(err_msg) = database.update_user(user).await {
            glib::g_warning!(crate::config::LOG_DOMAIN, "{err_msg}");
        }
    }

    pub async fn get_user(&self, database: &Database) -> User {
        let imp = self.imp();
        let user_id = i64::from(imp.settings.active_user_id());
        let user = &database.user(user_id).await.unwrap();
        user.clone()
    }
}

#[cfg(test)]
mod test {
    use super::Window;
    use crate::{core::Application, utils::init_gtk};
    use gtk::{gio, prelude::*};

    #[gtk::test]
    fn new() {
        init_gtk();

        let app = Application::new();
        app.set_application_id(Some("dev.Cogitri.Health.Tests.Window.New"));
        app.register(None::<&gio::Cancellable>).unwrap();
        Window::new(&app);
    }
}
