use crate::core::Database;
use chrono::Duration;
use gdk::subclass::prelude::ObjectSubclass;

mod imp {
    use crate::{
        core::{Database, Settings},
        model::Activity,
    };
    use chrono::Duration;
    use gio::ListModelExt;
    use glib::{subclass, Cast, StaticType};
    use gtk::subclass::prelude::*;
    use std::{
        cell::RefCell,
        convert::{TryFrom, TryInto},
    };

    #[derive(Debug)]
    pub struct ModelActivityMut {
        database: Option<Database>,
        vec: Vec<Activity>,
    }

    #[derive(Debug)]
    pub struct ModelActivity {
        inner: RefCell<ModelActivityMut>,
        settings: Settings,
    }

    impl ObjectSubclass for ModelActivity {
        const NAME: &'static str = "HealthModelActivity";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::ModelActivity;
        type Interfaces = (gio::ListModel,);

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(ModelActivityMut {
                    database: None,
                    vec: Vec::new(),
                }),
                settings: Settings::new(),
            }
        }
    }

    impl ObjectImpl for ModelActivity {}
    impl ListModelImpl for ModelActivity {
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
                .get(usize::try_from(position).unwrap())
                .map(|o| o.clone().upcast())
        }
    }

    impl ModelActivity {
        pub async fn reload(
            &self,
            obj: &super::ModelActivity,
            duration: Duration,
        ) -> Result<(), glib::Error> {
            let (database, previous_size) = {
                let inner = self.inner.borrow();
                (inner.database.clone(), inner.vec.len())
            };
            let new_vec = database
                .as_ref()
                .unwrap()
                .get_activities(Some(
                    chrono::Local::now()
                        .checked_sub_signed(duration)
                        .unwrap()
                        .into(),
                ))
                .await?;
            {
                self.inner.borrow_mut().vec = new_vec;
            }
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

        pub fn set_database(&self, database: Database) {
            self.inner.borrow_mut().database = Some(database);
        }
    }
}

glib::wrapper! {
    pub struct ModelActivity(ObjectSubclass<imp::ModelActivity>) @implements gio::ListModel;
}

impl ModelActivity {
    pub fn new(database: Database) -> Self {
        let o = glib::Object::new(&[]).expect("Failed to create ModelActivity");

        imp::ModelActivity::from_instance(&o).set_database(database);

        o
    }

    pub fn is_empty(&self) -> bool {
        imp::ModelActivity::from_instance(self).is_empty()
    }

    pub async fn reload(&self, duration: Duration) -> Result<(), glib::Error> {
        imp::ModelActivity::from_instance(self)
            .reload(self, duration)
            .await
    }
}
