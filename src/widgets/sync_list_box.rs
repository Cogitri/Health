use gdk::subclass::prelude::ObjectSubclass;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use crate::core::HealthSettings;
    use glib::subclass;
    use gtk::{prelude::*, subclass::prelude::*};
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/sync_list_box.ui")]
    pub struct HealthSyncListBox {
        pub parent_window: RefCell<Option<gtk::Window>>,

        #[template_child]
        pub google_fit_selected_image: TemplateChild<gtk::Image>,
        #[template_child]
        pub google_fit_start_sync_row: TemplateChild<gtk::ListBoxRow>,
        #[template_child]
        pub google_fit_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub google_fit_spinner: TemplateChild<gtk::Spinner>,
    }

    impl ObjectSubclass for HealthSyncListBox {
        const NAME: &'static str = "HealthSyncListBox";
        type ParentType = gtk::Widget;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthSyncListBox;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                parent_window: RefCell::new(None),
                google_fit_selected_image: TemplateChild::default(),
                google_fit_start_sync_row: TemplateChild::default(),
                google_fit_stack: TemplateChild::default(),
                google_fit_spinner: TemplateChild::default(),
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

    impl ObjectImpl for HealthSyncListBox {
        fn constructed(&self, _obj: &Self::Type) {
            if HealthSettings::new().get_sync_provider_setup_google_fit() {
                self.google_fit_selected_image.set_visible(true);
                self.google_fit_selected_image
                    .set_property_icon_name(Some("object-select-symbolic"));
                self.google_fit_stack
                    .set_visible_child(&self.google_fit_selected_image.get());
                self.google_fit_start_sync_row.set_activatable(false);
            }

            //self.google_fit_start_sync_row.connect_activated(|r| {});
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
    impl WidgetImpl for HealthSyncListBox {}
    impl ListBoxRowImpl for HealthSyncListBox {}

    impl HealthSyncListBox {
        fn open_sync_error(&self, errmsg: &str) {
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
        }
    }
}

glib::wrapper! {
    pub struct HealthSyncListBox(ObjectSubclass<imp::HealthSyncListBox>) @extends gtk::Widget, gtk::ListBoxRow;
}

impl HealthSyncListBox {
    pub fn new(parent_window: Option<gtk::Window>) -> Self {
        let s = glib::Object::new(&[]).expect("Failed to create HealthSyncListBox");

        imp::HealthSyncListBox::from_instance(&s)
            .parent_window
            .replace(parent_window);

        s
    }
}
