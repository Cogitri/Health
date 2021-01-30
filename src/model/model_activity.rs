use crate::{
    core::{HealthDatabase, HealthSettings},
    model::Activity,
};
use chrono::Duration;
use gdk::subclass::prelude::ObjectSubclass;

mod imp {
    use super::*;
    use gio::ListModelExt;
    use glib::{subclass, Cast, StaticType};
    use gtk::subclass::prelude::*;
    use std::{cell::RefCell, convert::TryInto};

    #[derive(Debug)]
    pub struct HealthModelActivityMut {
        database: Option<HealthDatabase>,
        vec: Vec<Activity>,
    }

    #[derive(Debug)]
    pub struct HealthModelActivity {
        inner: RefCell<HealthModelActivityMut>,
        settings: HealthSettings,
    }

    impl ObjectSubclass for HealthModelActivity {
        const NAME: &'static str = "HealthModelActivity";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthModelActivity;
        type Interfaces = (gio::ListModel,);

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(HealthModelActivityMut {
                    database: None,
                    vec: Vec::new(),
                }),
                settings: HealthSettings::new(),
            }
        }
    }

    impl ObjectImpl for HealthModelActivity {}
    impl ListModelImpl for HealthModelActivity {
        fn get_item_type(&self, _list_model: &Self::Type) -> glib::Type {
            Activity::static_type()
        }

        fn get_n_items(&self, _list_model: &Self::Type) -> u32 {
            self.inner.borrow().vec.len().try_into().unwrap()
        }

        fn get_item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.inner
                .borrow()
                .vec
                .get(position as usize)
                .map(|o| o.clone().upcast())
        }
    }

    impl HealthModelActivity {
        pub async fn reload(
            &self,
            obj: &super::HealthModelActivity,
            duration: Duration,
        ) -> Result<(), glib::Error> {
            let previous_size = self.inner.borrow().vec.len();
            let new_vec = self
                .inner
                .borrow()
                .database
                .as_ref()
                .unwrap()
                .get_activities(Some(
                    chrono::Local::now()
                        .checked_sub_signed(duration)
                        .unwrap()
                        .into(),
                ))
                .await?;
            self.inner.borrow_mut().vec = new_vec;
            obj.items_changed(
                0,
                previous_size.try_into().unwrap(),
                self.inner.borrow().vec.len().try_into().unwrap(),
            );
            Ok(())
        }

        pub fn is_empty(&self) -> bool {
            self.inner.borrow().vec.is_empty()
        }

        pub fn set_database(&self, database: HealthDatabase) {
            self.inner.borrow_mut().database = Some(database);
        }
    }
}

glib::wrapper! {
    pub struct HealthModelActivity(ObjectSubclass<imp::HealthModelActivity>) @implements gio::ListModel;
}

impl HealthModelActivity {
    pub fn new(database: HealthDatabase) -> Self {
        let o = glib::Object::new(&[]).expect("Failed to create HealthModelActivity");

        imp::HealthModelActivity::from_instance(&o).set_database(database);

        o
    }

    pub fn is_empty(&self) -> bool {
        imp::HealthModelActivity::from_instance(self).is_empty()
    }

    pub async fn reload(&self, duration: Duration) -> Result<(), glib::Error> {
        imp::HealthModelActivity::from_instance(self)
            .reload(self, duration)
            .await
    }
}
