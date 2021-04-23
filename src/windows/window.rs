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
    core::{i18n_f, settings::prelude::*, Database},
    sync::{
        google_fit::GoogleFitSyncProvider,
        new_db_receiver,
        sync_provider::{SyncProvider, SyncProviderError},
    },
    views::{ViewActivity, ViewSteps, ViewWeight},
    windows::{ActivityAddDialog, WeightAddDialog},
};
use glib::{clone, signal::Inhibit, subclass::prelude::*, Cast};
use gtk::prelude::*;
use gtk_macros::action;
use imp::ViewMode;
use std::collections::BTreeMap;

mod imp {
    use crate::{core::settings::prelude::*, views::View};
    use gio::Settings;
    use glib::SourceId;
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::{cell::RefCell, collections::BTreeMap};

    #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum ViewMode {
        STEPS,
        WEIGHT,
        ACTIVITIES,
    }

    #[derive(Debug)]
    pub struct WindowMut {
        pub current_height: i32,
        pub current_width: i32,
        pub current_view: ViewMode,
        pub sync_source_id: Option<SourceId>,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/window.ui")]
    pub struct Window {
        pub inner: RefCell<WindowMut>,
        pub settings: Settings,
        pub views: OnceCell<BTreeMap<ViewMode, View>>,

        #[template_child]
        pub add_data_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub error_infobar: TemplateChild<gtk::InfoBar>,
        #[template_child]
        pub error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub primary_menu_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "HealthWindow";
        type ParentType = adw::ApplicationWindow;
        type Type = super::Window;

        fn new() -> Self {
            Self {
                inner: RefCell::new(WindowMut {
                    current_height: 0,
                    current_width: 0,
                    current_view: ViewMode::STEPS,
                    sync_source_id: None,
                }),
                settings: Settings::instance(),
                views: OnceCell::new(),
                add_data_button: TemplateChild::default(),
                error_infobar: TemplateChild::default(),
                error_label: TemplateChild::default(),
                primary_menu_popover: TemplateChild::default(),
                stack: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if crate::config::APPLICATION_ID.ends_with("Devel") {
                obj.style_context().add_class("devel");

                // When in devel mode our application ID is different so we have to manually add the icon theme
                if let Some(icon_theme) = gtk::IconTheme::for_display(&obj.display()) {
                    icon_theme.add_resource_path("/dev/Cogitri/Health/icons");
                }
            }

            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/dev/Cogitri/Health/custom.css");
            gtk::StyleContext::add_provider_for_display(
                &obj.display(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            obj.connect_handlers();
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
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl Window {
    /// Create a new [Window].
    ///
    /// # Arguments
    /// * `app` - The application to use.
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P) -> Self {
        let o: Window =
            glib::Object::new(&[("application", app)]).expect("Failed to create Window");

        let obj = o.clone();
        gtk_macros::spawn!(async move {
            if let Err(e) = Database::instance().migrate().await {
                obj.show_error(&crate::core::i18n_f(
                    "Failed to migrate database to new version due to error {}",
                    &[&e.to_string()],
                ))
            }
            obj.create_views();
        });

        o
    }

    pub fn open_hamburger_menu(&self) {
        self.imp().primary_menu_popover.popup();
    }

    fn connect_handlers(&self) {
        let self_ = self.imp();

        self_
            .add_data_button
            .connect_clicked(clone!(@weak self as obj => move |_| {
                obj.handle_add_data_button_clicked();
            }));

        self_
            .error_infobar
            .connect_response(Self::handle_error_infobar_response);

        self_
            .stack
            .connect_property_visible_child_notify(clone!(@weak self as obj => move |_| {
                obj.handle_stack_property_visible_child_notify();
            }));

        self.connect_close_request(clone!(@weak self as obj => @default-panic, move |_| {
            obj.handle_close_request()
        }));

        self.connect_property_default_height_notify(clone!(@weak self as obj => move |_| {
            obj.handle_property_default_height_notify();
        }));

        self.connect_property_default_width_notify(clone!(@weak self as obj => move |_| {
            obj.handle_property_default_width_notify();
        }));
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
                if obj.is_fullscreen() {
                    obj.unfullscreen();
                } else {
                    obj.fullscreen();
                }
            })
        );
    }

    fn handle_add_data_button_clicked(&self) {
        let self_ = self.imp();

        let dialog = match self_.inner.borrow().current_view {
            ViewMode::ACTIVITIES | ViewMode::STEPS => {
                ActivityAddDialog::new(self.upcast_ref()).upcast::<gtk::Dialog>()
            }
            ViewMode::WEIGHT => WeightAddDialog::new(self.upcast_ref()).upcast::<gtk::Dialog>(),
        };
        dialog.present();
    }

    fn handle_close_request(&self) -> Inhibit {
        let self_ = self.imp();
        let mut inner = self_.inner.borrow_mut();

        self_.settings.set_window_is_maximized(self.is_maximized());
        self_.settings.set_window_height(inner.current_height);
        self_.settings.set_window_width(inner.current_width);

        if let Some(source_id) = inner.sync_source_id.take() {
            glib::source_remove(source_id);
        }

        Inhibit(false)
    }

    fn handle_error_infobar_response(bar: &gtk::InfoBar, response: gtk::ResponseType) {
        if response == gtk::ResponseType::Close {
            bar.set_revealed(false);
        }
    }

    fn handle_property_default_height_notify(&self) {
        self.imp().inner.borrow_mut().current_height = self.default_height();
    }

    fn handle_property_default_width_notify(&self) {
        self.imp().inner.borrow_mut().current_height = self.default_height();
    }

    fn handle_stack_property_visible_child_notify(&self) {
        let self_ = self.imp();
        let child_name = self_.stack.visible_child_name().map(|s| s.to_string());

        if child_name
            == self_
                .views
                .get()
                .unwrap()
                .get(&ViewMode::STEPS)
                .map(|s| s.widget_name().to_string())
        {
            self_.inner.borrow_mut().current_view = ViewMode::STEPS;
        } else if child_name
            == self_
                .views
                .get()
                .unwrap()
                .get(&ViewMode::WEIGHT)
                .map(|s| s.widget_name().to_string())
        {
            self_.inner.borrow_mut().current_view = ViewMode::WEIGHT;
        }
    }

    fn create_views(&self) {
        let self_ = self.imp();

        let mut views = BTreeMap::new();
        views.insert(ViewMode::ACTIVITIES, ViewActivity::new().upcast());
        views.insert(ViewMode::WEIGHT, ViewWeight::new().upcast());
        views.insert(ViewMode::STEPS, ViewSteps::new().upcast());
        self_.views.set(views).unwrap();

        for view in self_.views.get().unwrap().values() {
            let page = self_.stack.add_titled(
                view,
                Some(view.widget_name().as_str()),
                &view.view_title().unwrap(),
            );
            page.set_icon_name(&view.icon_name().unwrap());
        }

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
                clone!(@weak self as obj => @default-panic, move |v: Option<SyncProviderError>| {
                    if let Some(e) = v {
                        obj.show_error(&i18n_f(
                            "Couldn't sync Google Fit data due to error: {}",
                            &[&e.to_string()],
                        ));
                    }

                    glib::Continue(false)
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
        for (mode, view) in imp::Window::from_instance(self).views.get().unwrap() {
            match mode {
                imp::ViewMode::STEPS => {
                    let v = view.clone().downcast::<ViewSteps>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
                imp::ViewMode::WEIGHT => {
                    let v = view.clone().downcast::<ViewWeight>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
                imp::ViewMode::ACTIVITIES => {
                    let v = view.clone().downcast::<ViewActivity>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
            }
        }
    }

    fn imp(&self) -> &imp::Window {
        imp::Window::from_instance(self)
    }
}
