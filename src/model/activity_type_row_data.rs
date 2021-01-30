use gdk::subclass::prelude::ObjectSubclass;
use gtk::glib;

mod imp {
    use super::*;
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct HealthActivityTypeRowDataMut {
        pub id: &'static str,
        pub label: String,
    }

    #[derive(Debug)]
    pub struct HealthActivityTypeRowData {
        inner: RefCell<Option<HealthActivityTypeRowDataMut>>,
    }

    impl ObjectSubclass for HealthActivityTypeRowData {
        const NAME: &'static str = "HealthActivityTypeRowData";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthActivityTypeRowData;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for HealthActivityTypeRowData {}

    impl HealthActivityTypeRowData {
        pub fn set_inner(&self, inner: Option<HealthActivityTypeRowDataMut>) {
            self.inner.replace(inner);
        }

        pub fn get_id(&self) -> &'static str {
            self.inner.borrow().as_ref().unwrap().id
        }

        pub fn get_label(&self) -> String {
            self.inner.borrow().as_ref().unwrap().label.clone()
        }
    }
}

glib::wrapper! {
    pub struct HealthActivityTypeRowData(ObjectSubclass<imp::HealthActivityTypeRowData>);
}

impl HealthActivityTypeRowData {
    pub fn new(id: &'static str, label: &str) -> Self {
        let s = glib::Object::new(&[]).expect("Failed to create HealthActivityTypeRowData");

        imp::HealthActivityTypeRowData::from_instance(&s).set_inner(Some(
            imp::HealthActivityTypeRowDataMut {
                id,
                label: label.to_string(),
            },
        ));

        s
    }

    pub fn get_id(&self) -> &'static str {
        imp::HealthActivityTypeRowData::from_instance(self).get_id()
    }

    pub fn get_label(&self) -> String {
        imp::HealthActivityTypeRowData::from_instance(self).get_label()
    }
}
