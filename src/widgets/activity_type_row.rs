use crate::model::ActivityTypeRowData;
use gdk::subclass::prelude::ObjectSubclass;
use gtk::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/dev/Cogitri/Health/ui/activity_type_row.ui")]
    pub struct ActivityTypeRow {
        pub activity_type_id: RefCell<&'static str>,
        #[template_child]
        pub activity_type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub selected_image: TemplateChild<gtk::Image>,
    }

    impl ObjectSubclass for ActivityTypeRow {
        const NAME: &'static str = "HealthActivityTypeRow";
        type ParentType = gtk::ListBoxRow;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ActivityTypeRow;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                activity_type_id: RefCell::new(""),
                activity_type_label: TemplateChild::default(),
                selected_image: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self::Type>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ActivityTypeRow {}
    impl WidgetImpl for ActivityTypeRow {}
    impl ListBoxRowImpl for ActivityTypeRow {}

    impl ActivityTypeRow {
        pub fn get_id(&self) -> &'static str {
            *self.activity_type_id.borrow()
        }

        pub fn get_label(&self) -> String {
            self.activity_type_label.get_text().to_string()
        }

        pub fn get_selected(&self) -> bool {
            self.selected_image.get_visible()
        }

        pub fn set_id(&self, value: &'static str) {
            self.activity_type_id.replace(value);
        }

        pub fn set_label(&self, value: &str) {
            self.activity_type_label.set_text(value);
        }

        pub fn set_selected(&self, value: bool) {
            self.selected_image.set_visible(value);
        }
    }
}

glib::wrapper! {
    pub struct ActivityTypeRow(ObjectSubclass<imp::ActivityTypeRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl ActivityTypeRow {
    pub fn new(data: &ActivityTypeRowData, selected: bool) -> Self {
        let s = glib::Object::new(&[]).expect("Failed to create ActivityTypeRow");

        let self_ = imp::ActivityTypeRow::from_instance(&s);
        self_.set_id(data.get_id());
        self_.set_label(&data.get_label());
        self_.set_selected(selected);

        s
    }

    pub fn get_id(&self) -> &'static str {
        imp::ActivityTypeRow::from_instance(self).get_id()
    }

    pub fn get_label(&self) -> String {
        imp::ActivityTypeRow::from_instance(self).get_label()
    }

    pub fn get_selected(&self) -> bool {
        imp::ActivityTypeRow::from_instance(self).get_selected()
    }

    pub fn set_id(&self, value: &'static str) {
        imp::ActivityTypeRow::from_instance(self).set_id(value)
    }

    pub fn set_label(&self, value: &str) {
        imp::ActivityTypeRow::from_instance(self).set_label(value)
    }

    pub fn set_selected(&self, value: bool) {
        imp::ActivityTypeRow::from_instance(self).set_selected(value)
    }
}
