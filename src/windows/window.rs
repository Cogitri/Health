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

use crate::{
    core::{i18n_f, Database},
    sync::{google_fit::GoogleFitSyncProvider, new_db_receiver, sync_provider::SyncProvider},
    windows::DataAddDialog,
    ViewExt,
};
use gtk::{
    gio,
    glib::{self, clone, subclass::prelude::*, Cast},
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
        pub add_data_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub error_infobar: TemplateChild<gtk::InfoBar>,
        #[template_child]
        pub error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub primary_menu_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub view_home_page: TemplateChild<ViewHomePage>,
        #[template_child]
        pub enable_plugin_button: TemplateChild<gtk::Button>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

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

#[gtk::template_callbacks(value)]
impl Window {
    /// Create a new [Window].
    ///
    /// # Arguments
    /// * `app` - The application to use.
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
        let o: Self = glib::Object::new(&[("application", app)]).expect("Failed to create Window");

        let obj = o.clone();
        gtk_macros::spawn!(async move {
            if let Err(e) = Database::instance().migrate().await {
                obj.show_error(&crate::core::i18n_f(
                    "Failed to migrate database to new version due to error {}",
                    &[&e.to_string()],
                ))
            }
            obj.setup();
        });

        o
    }

    pub fn open_hamburger_menu(&self) {
        self.imp().primary_menu_popover.popup();
    }

    fn setup_actions(&self) {
        action!(
            self,
            "quit",
            clone!(@weak self as obj => move |_, _| {
                obj.destroy();
            })
        );
        action!(
            self,
            "hamburger-menu",
            clone!(@weak self as obj => move |_, _| {
                obj.open_hamburger_menu();
            })
        );
        action!(
            self,
            "fullscreen",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_fullscreen();
            })
        );
        action!(
            self,
            "disable-current-plugin",
            clone!(@weak self as obj => move |_, _| {
                obj.handle_disable_current_plugin();
            })
        );
        self.action_set_enabled("win.disable-current-plugin", false);
    }

    #[template_callback]
    fn handle_add_data_button_clicked(&self) {
        let dialog =
            DataAddDialog::new(self.upcast_ref(), self.imp().view_home_page.current_page())
                .upcast::<gtk::Dialog>();
        dialog.present();
    }

    #[template_callback]
    fn handle_back_button_clicked(&self) {
        let self_ = self.imp();
        self_.view_home_page.back();
        self.action_set_enabled("win.disable-current-plugin", false);
        self_.back_button.set_visible(false);
        self_.enable_plugin_button.set_visible(false);
    }

    #[template_callback]
    fn handle_close_request(&self) -> bool {
        let self_ = self.imp();
        let mut inner = self_.inner.borrow_mut();

        self_.settings.set_window_is_maximized(self.is_maximized());
        self_.settings.set_window_height(inner.current_height);
        self_.settings.set_window_width(inner.current_width);

        if let Some(source_id) = inner.sync_source_id.take() {
            source_id.remove();
        }

        false
    }

    #[template_callback]
    fn handle_disable_current_plugin(&self) {
        self.imp().view_home_page.disable_current_plugin();
        self.imp().view_home_page.back();
    }

    #[template_callback]
    fn handle_enable_plugin_button_clicked(&self) {
        let self_ = self.imp();
        self_.view_home_page.enable_current_plugin();
        self_.view_home_page.back();
        self_.enable_plugin_button.set_visible(false);
    }

    #[template_callback]
    fn handle_error_infobar_response(bar: &gtk::InfoBar, response: gtk::ResponseType) {
        if response == gtk::ResponseType::Close {
            bar.set_revealed(false);
        }
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

    #[template_callback]
    fn handle_view_changed(&self) {
        let self_ = self.imp();
        let is_enabled = self_.view_home_page.is_current_plugin_enabled();
        self.action_set_enabled("win.disable-current-plugin", is_enabled);
        self_.enable_plugin_button.set_visible(!is_enabled);
        self_.back_button.set_visible(true)
    }

    fn setup(&self) {
        self.update();
        self.sync_data();

        // FIXME: Allow setting custom sync interval
        glib::timeout_add_seconds_local(
            60 * 5,
            clone!(@weak self as obj => @default-panic, move || {
                obj.sync_data();

                glib::Continue(true)
            }),
        );
    }

    fn handle_sync_data_error_received(&self, err_opt: Option<anyhow::Error>) -> glib::Continue {
        if let Some(e) = err_opt {
            self.show_error(&i18n_f(
                "Couldn't sync Google Fit data due to error: {}",
                &[&e.to_string()],
            ));
        }

        glib::Continue(false)
    }

    /// Display an error in a non-intrusive way.
    fn show_error(&self, err_msg: &str) {
        let self_ = self.imp();

        glib::g_warning!(crate::config::LOG_DOMAIN, "{}", err_msg);
        self_.error_label.set_text(err_msg);
        self_.error_infobar.set_revealed(true);
    }

    fn sync_data(&self) {
        let self_ = self.imp();

        if self_.settings.sync_provider_setup_google_fit() {
            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let db_sender = new_db_receiver();

            receiver.attach(
                None,
                clone!(@weak self as obj => @default-panic, move |v: Option<anyhow::Error>| {
                    obj.handle_sync_data_error_received(v)
                }),
            );

            std::thread::spawn(move || {
                let mut sync_proxy = GoogleFitSyncProvider::new(db_sender);
                if let Err(e) = sync_proxy.sync_data() {
                    sender.send(Some(e)).unwrap();
                } else {
                    sender.send(None).unwrap();
                }
            });
        }
    }

    pub fn update(&self) {
        let view = self.imp().view_home_page.get();
        gtk_macros::spawn!(clone!(@weak view => async move {
            view.update().await.unwrap();
        }));
    }

    fn imp(&self) -> &imp::Window {
        imp::Window::from_instance(self)
    }
}
