use crate::{
    core::Database,
    views::{ViewActivity, ViewSteps, ViewWeight},
};
use glib::subclass::types::ObjectSubclass;
use glib::Cast;
use gtk::prelude::*;

mod imp {
    use crate::{
        core::{Database, Settings},
        sync::{
            google_fit::GoogleFitSyncProvider,
            new_db_receiver,
            sync_provider::{SyncProvider, SyncProviderError},
        },
        views::{View, ViewActivity, ViewSteps, ViewWeight},
        windows::{ActivityAddDialog, WeightAddDialog},
    };
    use glib::{clone, signal::Inhibit, subclass, SourceId};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum ViewMode {
        STEPS,
        WEIGHT,
        ACTIVITIES,
    }

    #[derive(Debug)]
    pub struct WindowMut {
        current_height: i32,
        current_width: i32,
        current_view: ViewMode,
        sync_source_id: Option<SourceId>,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/window.ui")]
    pub struct Window {
        pub db: OnceCell<Database>,
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

    impl ObjectSubclass for Window {
        const NAME: &'static str = "HealthWindow";
        type ParentType = adw::ApplicationWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::Window;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                db: OnceCell::new(),
                inner: RefCell::new(WindowMut {
                    current_height: 0,
                    current_width: 0,
                    current_view: ViewMode::STEPS,
                    sync_source_id: None,
                }),
                settings: Settings::new(),
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

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if crate::config::APPLICATION_ID.ends_with("Devel") {
                obj.get_style_context().add_class("devel");

                // When in devel mode our application ID is different so we have to manually add the icon theme
                if let Some(icon_theme) = gtk::IconTheme::get_for_display(&obj.get_display()) {
                    icon_theme.add_resource_path("/dev/Cogitri/Health/icons");
                }
            }

            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/dev/Cogitri/Health/custom.css");
            gtk::StyleContext::add_provider_for_display(
                &obj.get_display(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            self.connect_handlers(obj);
        }
    }

    impl Window {
        pub fn show_error(&self, err_msg: &str) {
            glib::g_warning!(crate::config::LOG_DOMAIN, "{}", err_msg);
            self.error_label.set_text(err_msg);
            self.error_infobar.set_revealed(true);
        }

        fn connect_handlers(&self, obj: &super::Window) {
            self.error_infobar.connect_response(|bar, response| {
                if response == gtk::ResponseType::Close {
                    bar.set_revealed(false);
                }
            });
            self.stack
                .connect_property_visible_child_notify(clone!(@weak obj => move |s| {
                    let child_name = s.get_visible_child_name().map(|s| s.to_string());
                    let self_ = Window::from_instance(&obj);

                    if child_name == self_.views.get().unwrap().get(&ViewMode::STEPS).and_then(|s| s.get_name()).map(|s| s.to_string()) {
                        self_.inner.borrow_mut().current_view = ViewMode::STEPS;
                    } else if child_name == self_.views.get().unwrap().get(&ViewMode::WEIGHT).and_then(|s| s.get_name()).map(|s| s.to_string()) {
                        self_.inner.borrow_mut().current_view = ViewMode::WEIGHT;
                    }
                }));
            self.add_data_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let self_ = Window::from_instance(&obj);
                    let db = self_.db.get().unwrap().clone();

                    let dialog = match self_.inner.borrow().current_view {
                        ViewMode::ACTIVITIES | ViewMode::STEPS => ActivityAddDialog::new(db, obj.upcast_ref()).upcast::<gtk::Dialog>(),
                        ViewMode::WEIGHT => WeightAddDialog::new(db, obj.upcast_ref()).upcast::<gtk::Dialog>(),
                    };
                    dialog.present();
                }));

            obj.connect_property_default_height_notify(move |w| {
                Window::from_instance(w).inner.borrow_mut().current_height =
                    w.get_property_default_height();
            });
            obj.connect_property_default_width_notify(move |w| {
                Window::from_instance(w).inner.borrow_mut().current_width =
                    w.get_property_default_width();
            });
            obj.connect_close_request(|w| {
                let self_ = Window::from_instance(w);
                let mut inner = self_.inner.borrow_mut();

                self_
                    .settings
                    .set_window_is_maximized(w.get_property_maximized());
                self_.settings.set_window_height(inner.current_height);
                self_.settings.set_window_width(inner.current_width);

                if let Some(source_id) = inner.sync_source_id.take() {
                    glib::source_remove(source_id);
                }

                Inhibit(false)
            });
        }

        pub fn set_db(&self, obj: &super::Window, db: Database) {
            self.db.set(db.clone()).unwrap();

            let mut views = BTreeMap::new();
            views.insert(ViewMode::ACTIVITIES, ViewActivity::new(db.clone()).upcast());
            views.insert(ViewMode::WEIGHT, ViewWeight::new(db.clone()).upcast());
            views.insert(ViewMode::STEPS, ViewSteps::new(db).upcast());
            self.views.set(views).unwrap();

            for view in self.views.get().unwrap().values() {
                let page = self.stack.add_titled(
                    view,
                    view.get_name().map(|s| s.to_string()).as_deref(),
                    &view.get_view_title().unwrap(),
                );
                page.set_icon_name(&view.get_icon_name().unwrap());
            }

            obj.update();
            self.sync_data(obj);

            // FIXME: Allow setting custom sync interval
            glib::timeout_add_seconds_local(
                60 * 5,
                clone!(@weak obj => move || {
                    let self_ = Window::from_instance(&obj);
                    self_.sync_data(&obj);

                    glib::Continue(true)
                }),
            );
        }

        fn sync_data(&self, obj: &super::Window) {
            if self.settings.get_sync_provider_setup_google_fit() {
                let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                let db_sender = new_db_receiver(self.db.get().unwrap().clone());

                receiver.attach(
                    None,
                    clone!(@weak obj => move |v: Option<SyncProviderError>| {
                        if let Some(e) = v {
                            Window::from_instance(&obj).show_error(&e.to_string());
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
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl Window {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, db: Database) -> Self {
        let o = glib::Object::new(&[("application", app)]).expect("Failed to create Window");

        imp::Window::from_instance(&o).set_db(&o, db);

        o
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

    pub fn open_hamburger_menu(&self) {
        imp::Window::from_instance(self)
            .primary_menu_popover
            .popup();
    }
}
