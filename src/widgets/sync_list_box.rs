use crate::core::Database;
use glib::subclass::types::ObjectSubclass;

mod imp {
    use crate::{
        core::{Database, Settings},
        sync::{
            google_fit::GoogleFitSyncProvider,
            new_db_receiver,
            sync_provider::{SyncProvider, SyncProviderError},
        },
    };
    use glib::{clone, g_warning, subclass};
    use gtk::{prelude::*, subclass::prelude::*, CompositeTemplate};
    use gtk_macros::spawn;
    use once_cell::unsync::OnceCell;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/sync_list_box.ui")]
    pub struct SyncListBox {
        pub database: OnceCell<Database>,
        pub parent_window: RefCell<Option<gtk::Window>>,

        #[template_child]
        pub google_fit_selected_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub google_fit_start_sync_row: TemplateChild<gtk::ListBoxRow>,
        #[template_child]
        pub google_fit_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub google_fit_spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub sync_list_box: TemplateChild<gtk::ListBox>,
    }

    impl ObjectSubclass for SyncListBox {
        const NAME: &'static str = "HealthSyncListBox";
        type ParentType = gtk::Widget;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::SyncListBox;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                database: OnceCell::new(),
                parent_window: RefCell::new(None),
                google_fit_selected_image: TemplateChild::default(),
                google_fit_start_sync_row: TemplateChild::default(),
                google_fit_stack: TemplateChild::default(),
                google_fit_spinner: TemplateChild::default(),
                sync_list_box: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SyncListBox {
        fn constructed(&self, obj: &Self::Type) {
            if Settings::new().get_sync_provider_setup_google_fit() {
                self.google_fit_selected_image.set_visible(true);
                self.google_fit_selected_image
                    .set_property_icon_name(Some("object-select-symbolic"));
                self.google_fit_stack
                    .set_visible_child(&self.google_fit_selected_image.get());
                self.google_fit_start_sync_row.set_activatable(false);
            }

            self.connect_handlers(obj);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::object(
                    "parent-window",
                    "parent-window",
                    "parent-window",
                    gtk::Window::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.get_name() {
                "parent-window" => {
                    self.parent_window.replace(value.get().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn get_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            pspec: &glib::ParamSpec,
        ) -> glib::Value {
            match pspec.get_name() {
                "parent-window" => self.parent_window.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for SyncListBox {}
    impl ListBoxRowImpl for SyncListBox {}

    impl SyncListBox {
        fn connect_handlers(&self, obj: &super::SyncListBox) {
            self.sync_list_box
                .connect_row_activated(clone!(@weak obj => move |list_box, row| {
                    let self_ = SyncListBox::from_instance(&obj);
                    if (row == &self_.google_fit_start_sync_row.get()) {
                        self_.google_fit_stack.set_visible(true);
                        self_.google_fit_spinner.set_visible(true);
                        self_.google_fit_spinner.set_spinning(true);
                        self_.google_fit_start_sync_row.set_activatable(false);
                        self_.google_fit_stack.set_visible_child(&self_.google_fit_spinner.get());

                        let (sender, receiver) = glib::MainContext::channel::<Result<GoogleFitSyncProvider, SyncProviderError>>(glib::PRIORITY_DEFAULT);
                        let db_sender = new_db_receiver(self_.database.get().unwrap().clone());

                        receiver.attach(None, clone!(@weak obj => move |sync_provider| {
                            if let Ok(provider) = sync_provider {
                                spawn!(async move {
                                    // TODO: Start importing data
                                    let self_ = SyncListBox::from_instance(&obj);
                                    self_.google_fit_selected_image.set_visible(true);
                                    self_.google_fit_spinner.set_spinning(false);
                                    self_.google_fit_stack.set_visible_child(&self_.google_fit_selected_image.get());
                                });
                            } else {
                                let self_ = SyncListBox::from_instance(&obj);

                                self_.google_fit_selected_image.set_property_icon_name(Some("network-error-symbolic"));
                                self_.google_fit_selected_image.set_visible(true);
                                self_.google_fit_spinner.set_spinning(false);
                                self_.google_fit_stack.set_visible_child(&self_.google_fit_selected_image.get());

                                self_.open_sync_error(&sync_provider.err().unwrap().to_string());
                            }

                            glib::Continue(false)
                        }));

                        std::thread::spawn(move || {
                            let mut sync_provider = GoogleFitSyncProvider::new(db_sender);
                            if let Err(e) = sync_provider.initial_authenticate() {
                                sender.send(Err(e)).unwrap();
                            } else {
                                if let Err(e) = sync_provider.initial_import() {
                                    sender.send(Err(e)).unwrap();
                                }

                                sender.send(Ok(sync_provider)).unwrap();
                            }
                        });
                    }
                }));
        }

        fn open_sync_error(&self, errmsg: &str) {
            g_warning!(crate::config::LOG_DOMAIN, "{}", errmsg);

            let dialog = gtk::MessageDialog::new(
                self.parent_window.borrow().as_ref(),
                gtk::DialogFlags::DESTROY_WITH_PARENT | gtk::DialogFlags::MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                errmsg,
            );
            dialog.connect_response(|d, _| {
                d.destroy();
            });
            dialog.show();
        }
    }
}

glib::wrapper! {
    pub struct SyncListBox(ObjectSubclass<imp::SyncListBox>) @extends gtk::Widget, gtk::ListBoxRow;
}

impl SyncListBox {
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        let s = glib::Object::new(&[]).expect("Failed to create SyncListBox");

        imp::SyncListBox::from_instance(&s)
            .parent_window
            .replace(parent_window);

        s
    }

    pub fn set_database(&self, database: Database) {
        imp::SyncListBox::from_instance(self)
            .database
            .set(database)
            .unwrap()
    }
}
