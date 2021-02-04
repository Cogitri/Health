use glib::subclass::types::ObjectSubclass;

mod imp {
    use glib::subclass;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct ActivityTypeRowDataMut {
        pub id: &'static str,
        pub label: String,
    }

    #[derive(Debug)]
    pub struct ActivityTypeRowData {
        inner: RefCell<Option<ActivityTypeRowDataMut>>,
    }

    impl ObjectSubclass for ActivityTypeRowData {
        const NAME: &'static str = "HealthActivityTypeRowData";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ActivityTypeRowData;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for ActivityTypeRowData {}

    impl ActivityTypeRowData {
        pub fn set_inner(&self, inner: Option<ActivityTypeRowDataMut>) {
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
    pub struct ActivityTypeRowData(ObjectSubclass<imp::ActivityTypeRowData>);
}

impl ActivityTypeRowData {
    pub fn new(id: &'static str, label: &str) -> Self {
        let s = glib::Object::new(&[]).expect("Failed to create ActivityTypeRowData");

        imp::ActivityTypeRowData::from_instance(&s).set_inner(Some(imp::ActivityTypeRowDataMut {
            id,
            label: label.to_string(),
        }));

        s
    }

    pub fn get_id(&self) -> &'static str {
        imp::ActivityTypeRowData::from_instance(self).get_id()
    }

    pub fn get_label(&self) -> String {
        imp::ActivityTypeRowData::from_instance(self).get_label()
    }
}
