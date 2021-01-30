use crate::{
    core::HealthDatabase,
    views::{HealthViewActivity, HealthViewSteps, HealthViewWeight},
};
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

mod imp {
    use super::*;
    use crate::{
        core::HealthSettings,
        views::HealthView,
        windows::{HealthActivityAddDialog, HealthWeightAddDialog},
    };
    use glib::{clone, signal::Inhibit, subclass, SourceId};
    use gtk::subclass::prelude::*;
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
    pub struct HealthWindowMut {
        current_height: i32,
        current_width: i32,
        current_view: ViewMode,
        sync_source_id: Option<SourceId>,
    }

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/window.ui")]
    pub struct HealthWindow {
        pub db: OnceCell<HealthDatabase>,
        pub inner: RefCell<HealthWindowMut>,
        pub settings: HealthSettings,
        pub views: OnceCell<BTreeMap<ViewMode, HealthView>>,

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

    impl ObjectSubclass for HealthWindow {
        const NAME: &'static str = "HealthWindow";
        type ParentType = adw::ApplicationWindow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthWindow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                db: OnceCell::new(),
                inner: RefCell::new(HealthWindowMut {
                    current_height: 0,
                    current_width: 0,
                    current_view: ViewMode::STEPS,
                    sync_source_id: None,
                }),
                settings: HealthSettings::new(),
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

    impl ObjectImpl for HealthWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if crate::config::APPLICATION_ID.ends_with("Devel") {
                obj.get_style_context().add_class("devel");

                // When in devel mode our application ID is different so we have to manually add the icon theme
                gtk::IconTheme::get_for_display(&obj.get_display())
                    .map(|i| i.add_resource_path("/dev/Cogitri/Health/icons"));
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

    impl HealthWindow {
        pub fn show_error(&self, err_msg: &str) {
            glib::g_warning!(crate::config::LOG_DOMAIN, "{}", err_msg);
            self.error_label.set_text(err_msg);
            self.error_infobar.set_revealed(true);
        }

        fn connect_handlers(&self, obj: &super::HealthWindow) {
            self.error_infobar.connect_response(|bar, response| {
                if response == gtk::ResponseType::Close {
                    bar.set_revealed(false);
                }
            });
            self.stack
                .connect_property_visible_child_notify(clone!(@weak obj => move |s| {
                    let child_name = s.get_visible_child_name().map(|s| s.to_string());
                    let self_ = HealthWindow::from_instance(&obj);

                    if child_name == self_.views.get().unwrap().get(&ViewMode::STEPS).and_then(|s| s.get_name()).map(|s| s.to_string()) {
                        self_.inner.borrow_mut().current_view = ViewMode::STEPS;
                    } else if child_name == self_.views.get().unwrap().get(&ViewMode::WEIGHT).and_then(|s| s.get_name()).map(|s| s.to_string()) {
                        self_.inner.borrow_mut().current_view = ViewMode::WEIGHT;
                    }
                }));
            self.add_data_button
                .connect_clicked(clone!(@weak obj => move |_| {
                    let self_ = HealthWindow::from_instance(&obj);
                    let db = self_.db.get().unwrap().clone();

                    let dialog = match self_.inner.borrow().current_view {
                        ViewMode::ACTIVITIES | ViewMode::STEPS => HealthActivityAddDialog::new(db, obj.upcast_ref()).upcast::<gtk::Dialog>(),
                        ViewMode::WEIGHT => HealthWeightAddDialog::new(db, obj.upcast_ref()).upcast::<gtk::Dialog>(),
                    };
                    dialog.present();
                }));

            obj.connect_property_default_height_notify(move |w| {
                HealthWindow::from_instance(w)
                    .inner
                    .borrow_mut()
                    .current_height = w.get_property_default_height();
            });
            obj.connect_property_default_width_notify(move |w| {
                HealthWindow::from_instance(w)
                    .inner
                    .borrow_mut()
                    .current_width = w.get_property_default_width();
            });
            obj.connect_close_request(|w| {
                let self_ = HealthWindow::from_instance(w);
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

        pub fn set_db(&self, obj: &super::HealthWindow, db: HealthDatabase) {
            self.db.set(db.clone()).unwrap();

            let mut views = BTreeMap::new();
            views.insert(
                ViewMode::ACTIVITIES,
                HealthViewActivity::new(db.clone()).upcast(),
            );
            views.insert(ViewMode::WEIGHT, HealthViewWeight::new(db.clone()).upcast());
            views.insert(ViewMode::STEPS, HealthViewSteps::new(db.clone()).upcast());
            self.views.set(views).unwrap();

            for (_, view) in self.views.get().unwrap() {
                let page = self.stack.add_titled(
                    view,
                    view.get_name().map(|s| s.to_string()).as_deref(),
                    &view.get_view_title().unwrap(),
                );
                page.set_icon_name(&view.get_icon_name().unwrap());
            }

            obj.update();
        }
    }

    impl WidgetImpl for HealthWindow {}
    impl WindowImpl for HealthWindow {}
    impl ApplicationWindowImpl for HealthWindow {}
    impl adw::subclass::application_window::AdwApplicationWindowImpl for HealthWindow {}
}

glib::wrapper! {
    pub struct HealthWindow(ObjectSubclass<imp::HealthWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow, @implements gio::ActionMap, gio::ActionGroup;
}

impl HealthWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(app: &P, db: HealthDatabase) -> Self {
        let o = glib::Object::new(&[("application", app)]).expect("Failed to create HealthWindow");

        imp::HealthWindow::from_instance(&o).set_db(&o, db);

        o
    }

    pub fn update(&self) {
        for (mode, view) in imp::HealthWindow::from_instance(self).views.get().unwrap() {
            match mode {
                imp::ViewMode::STEPS => {
                    let v = view.clone().downcast::<HealthViewSteps>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
                imp::ViewMode::WEIGHT => {
                    let v = view.clone().downcast::<HealthViewWeight>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
                imp::ViewMode::ACTIVITIES => {
                    let v = view.clone().downcast::<HealthViewActivity>().unwrap();
                    glib::MainContext::default().spawn_local(async move {
                        v.update().await;
                    });
                }
            }
        }
    }

    pub fn open_hamburger_menu(&self) {
        imp::HealthWindow::from_instance(self)
            .primary_menu_popover
            .popup();
    }
}
