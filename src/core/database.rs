use crate::model::{Activity, Steps, Weight};
use chrono::{DateTime, FixedOffset};
use gdk::subclass::prelude::ObjectSubclass;
use glib::ObjectExt;

mod imp {
    use crate::model::{Activity, ActivityType, Steps, Weight};
    use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
    use glib::subclass::{self, Signal};
    use glib::ObjectExt;
    use gtk::subclass::prelude::*;
    use num_traits::cast::{FromPrimitive, ToPrimitive};
    use std::{
        cell::RefCell,
        convert::{TryFrom, TryInto},
        path::Path,
    };
    use uom::si::{
        f32::{Length, Mass},
        length::meter,
        mass::kilogram,
    };

    #[derive(Debug)]
    pub struct DatabaseMut {
        connection: tracker::SparqlConnection,
        manager: tracker::NamespaceManager,
    }

    #[derive(Debug)]
    pub struct Database {
        inner: RefCell<Option<DatabaseMut>>,
    }

    impl ObjectSubclass for Database {
        const NAME: &'static str = "HealthDatabase";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::Database;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for Database {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("activities-updated", &[], glib::Type::Unit).build(),
                    Signal::builder("weights-updated", &[], glib::Type::Unit).build(),
                ]
            });

            SIGNALS.as_ref()
        }
    }

    impl Database {
        pub fn connect(
            &self,
            ontology_path: Option<std::path::PathBuf>,
            store_path: Option<std::path::PathBuf>,
        ) -> Result<(), glib::Error> {
            let mut store_path = store_path.unwrap_or_else(glib::get_user_data_dir);
            store_path.push("health");

            let mut ontology_path =
                ontology_path.unwrap_or_else(|| Path::new(crate::config::PKGDATADIR).to_path_buf());
            ontology_path.push("ontology");

            let manager = tracker::NamespaceManager::new();
            manager.add_prefix("health", "https://gitlab.gnome.org/World/health#");

            self.inner.replace(Some(DatabaseMut {
                connection: tracker::SparqlConnection::new(
                    tracker::SparqlConnectionFlags::NONE,
                    Some(&gio::File::new_for_path(store_path)),
                    Some(&gio::File::new_for_path(ontology_path)),
                    None::<&gio::Cancellable>,
                )?,
                manager,
            }));

            Ok(())
        }

        pub async fn get_activities(
            &self,
            date_opt: Option<DateTime<FixedOffset>>,
        ) -> Result<Vec<Activity>, glib::Error> {
            let cursor = if let Some(date) = date_opt {
                let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
                connection.query_async_future(&format!("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL {{ ?datapoint health:calories_burned ?calories_burned . }} OPTIONAL {{ ?datapoint health:distance ?distance . }} OPTIONAL {{ ?datapoint health:hearth_rate_avg ?heart_rate_avg . }} OPTIONAL {{ ?datapoint health:hearth_rate_min ?heart_rate_min . }} OPTIONAL {{ ?datapoint health:hearth_rate_max ?heart_rate_max . }} OPTIONAL {{ ?datapoint health:steps ?steps . }} OPTIONAL {{ ?datapoint health:minutes ?minutes }} FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY DESC(?date)", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
            } else {
                let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
                connection.query_async_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?
            };

            let mut ret = Vec::new();
            while let Ok(true) = cursor.next_async_future().await {
                let activity = Activity::new();

                for i in 0..cursor.get_n_columns() {
                    match cursor.get_variable_name(i).unwrap().as_str() {
                        "id" => {
                            activity.set_activity_type(
                                ActivityType::from_i64(cursor.get_integer(i)).unwrap(),
                            );
                        }
                        "date" => {
                            let datetime = DateTime::parse_from_rfc3339(
                                cursor.get_string(i).0.unwrap().as_str(),
                            );
                            if datetime.is_err() {
                                // Migrate from previous date format
                                let ndt = NaiveDate::parse_from_str(
                                    cursor.get_string(i).0.unwrap().as_str(),
                                    "%Y-%m-%d",
                                )
                                .unwrap()
                                .and_hms(0, 0, 0);
                                activity.set_date(DateTime::<Utc>::from_utc(ndt, Utc).into());
                            } else {
                                activity.set_date(datetime.unwrap());
                            }
                        }
                        "calories_burned" => {
                            activity.set_calories_burned(Some(
                                cursor.get_integer(i).try_into().unwrap(),
                            ));
                        }
                        "distance" => {
                            activity.set_distance(Some(Length::new::<meter>(
                                cursor.get_integer(i) as f32,
                            )));
                        }
                        "heart_rate_avg" => {
                            activity.set_heart_rate_avg(Some(
                                cursor.get_integer(i).try_into().unwrap(),
                            ));
                        }
                        "heart_rate_max" => {
                            activity.set_heart_rate_max(Some(
                                cursor.get_integer(i).try_into().unwrap(),
                            ));
                        }
                        "heart_rate_min" => {
                            activity.set_heart_rate_min(Some(
                                cursor.get_integer(i).try_into().unwrap(),
                            ));
                        }
                        "minutes" => {
                            activity.set_duration(Duration::minutes(cursor.get_integer(i)));
                        }
                        "steps" => {
                            activity.set_steps(Some(cursor.get_integer(i).try_into().unwrap()));
                        }
                        _ => unimplemented!(),
                    }
                }

                ret.push(activity);
            }

            Ok(ret)
        }

        pub async fn get_steps(
            &self,
            date: DateTime<FixedOffset>,
        ) -> Result<Vec<Steps>, glib::Error> {
            let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
            let cursor = connection.query_async_future(&format!("SELECT ?date ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_date ?date ; health:steps ?steps . FILTER  (?date >= '{}'^^xsd:dateTime)}}", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
            let mut hashmap = std::collections::HashMap::new();

            while let Ok(true) = cursor.next_async_future().await {
                let datetime =
                    DateTime::parse_from_rfc3339(cursor.get_string(0).0.unwrap().as_str());
                let date = if datetime.is_err() {
                    // Migrate from previous date format
                    let ndt = NaiveDate::parse_from_str(
                        cursor.get_string(0).0.unwrap().as_str(),
                        "%Y-%m-%d",
                    )
                    .unwrap()
                    .and_hms(0, 0, 0);
                    DateTime::<Utc>::from_utc(ndt, Utc).into()
                } else {
                    datetime.unwrap()
                };
                hashmap.insert(
                    date,
                    hashmap.get(&date).unwrap_or(&0)
                        + u32::try_from(cursor.get_integer(1)).unwrap(),
                );
            }

            let mut v: Vec<Steps> = hashmap
                .drain()
                .map(|(date, steps)| Steps::new(date, steps))
                .collect();

            v.sort_by(|a, b| a.date.cmp(&b.date));

            Ok(v)
        }

        pub async fn get_weights(
            &self,
            date_opt: Option<DateTime<FixedOffset>>,
        ) -> Result<Vec<Weight>, glib::Error> {
            let cursor = if let Some(date) = date_opt {
                let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
                connection.query_async_future(&format!("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY ?date", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
            } else {
                let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
                connection.query_async_future("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . }} ORDER BY ?date").await?
            };
            let mut ret = Vec::new();

            while let Ok(true) = cursor.next_async_future().await {
                ret.push(Weight::new(
                    DateTime::<Utc>::from_utc(
                        NaiveDate::parse_from_str(
                            cursor.get_string(0).0.unwrap().as_str(),
                            "%Y-%m-%d",
                        )
                        .unwrap()
                        .and_hms(0, 0, 0),
                        Utc,
                    )
                    .into(),
                    Mass::new::<kilogram>(cursor.get_double(1) as f32),
                ));
            }

            Ok(ret)
        }

        pub async fn get_weight_exists_on_date(
            &self,
            date: DateTime<FixedOffset>,
        ) -> Result<bool, glib::Error> {
            let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
            let cursor = connection.query_async_future(&format!("ASK {{ ?datapoint a health:WeightMeasurement ; health:weight_date '{}'^^xsd:date; health:weight ?weight . }}", date.date().format("%Y-%m-%d"))).await?;

            assert!(cursor.next_async_future().await?);

            return Ok(cursor.get_boolean(0));
        }

        pub async fn reset(&self) -> Result<(), glib::Error> {
            let connection = { self.inner.borrow().as_ref().unwrap().connection.clone() };
            connection
                .update_async_future("DELETE WHERE { ?datapoint a health:WeightMeasurement }")
                .await?;
            connection
                .update_async_future("DELETE WHERE { ?datapoint a health:Activity }")
                .await?;

            Ok(())
        }

        pub async fn import_steps(
            &self,
            obj: &super::Database,
            steps: &[Steps],
        ) -> Result<(), glib::Error> {
            if steps.is_empty() {
                return Ok(());
            }

            let (connection, manager) = {
                let inner_ref = self.inner.borrow();
                let inner = inner_ref.as_ref().unwrap();
                (inner.connection.clone(), inner.manager.clone())
            };

            for s in steps {
                let resource = tracker::Resource::new(None);
                resource.set_uri("rdf:type", "health:Activity");
                resource.set_string(
                    "health:activity_date",
                    &format!("{}", s.date.date().format("%Y-%m-%d")),
                );
                resource.set_int64("health:steps", s.steps.into());
                resource.set_int64(
                    "health:activity_id",
                    ActivityType::Walking.to_i64().unwrap(),
                );
                // FIXME: Set correct minutes here
                resource.set_int64("health:minutes", 0);

                connection
                    .update_async_future(
                        resource
                            .print_sparql_update(Some(&manager), None)
                            .unwrap()
                            .as_str(),
                    )
                    .await?;
            }

            obj.emit("activities-updated", &[]).unwrap();
            Ok(())
        }

        pub async fn import_weights(
            &self,
            obj: &super::Database,
            weights: &[Weight],
        ) -> Result<(), glib::Error> {
            if weights.is_empty() {
                return Ok(());
            }

            let (connection, manager) = {
                let inner_ref = self.inner.borrow();
                let inner = inner_ref.as_ref().unwrap();
                (inner.connection.clone(), inner.manager.clone())
            };

            for w in weights {
                let resource = tracker::Resource::new(None);
                resource.set_uri("rdf:type", "health:WeightMeasurement");
                resource.set_string(
                    "health:weight_date",
                    &format!("{}", w.date.date().format("%Y-%m-%d")),
                );
                resource.set_double("health:weight", w.weight.get::<kilogram>().into());

                connection
                    .update_async_future(
                        resource
                            .print_sparql_update(Some(&manager), None)
                            .unwrap()
                            .as_str(),
                    )
                    .await?;
            }

            obj.emit("weights-updated", &[]).unwrap();
            Ok(())
        }

        pub async fn save_activity(
            &self,
            obj: &super::Database,
            activity: Activity,
        ) -> Result<(), glib::Error> {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:Activity");
            resource.set_string(
                "health:activity_date",
                &format!("{}", activity.get_date().date().format("%Y-%m-%d")),
            );
            resource.set_int64(
                "health:activity_id",
                activity.get_activity_type().to_u32().unwrap().into(),
            );

            if let Some(c) = activity.get_calories_burned() {
                resource.set_int64("health:calories_burned", c.into());
            }
            if let Some(d) = activity.get_distance() {
                resource.set_int64(
                    "health:distance",
                    d.get::<uom::si::length::kilometer>() as i64,
                );
            }
            if let Some(avg) = activity.get_heart_rate_avg() {
                resource.set_int64("health:hearth_rate_avg", avg.into());
            }
            if let Some(max) = activity.get_heart_rate_max() {
                resource.set_int64("health:hearth_rate_max", max.into());
            }
            if let Some(min) = activity.get_heart_rate_min() {
                resource.set_int64("health:hearth_rate_min", min.into());
            }
            if activity.get_duration().num_minutes() != 0 {
                resource.set_int64("health:minutes", activity.get_duration().num_minutes());
            }
            if let Some(s) = activity.get_steps() {
                resource.set_int64("health:steps", s.into());
            }

            let (connection, manager) = {
                let inner_ref = self.inner.borrow();
                let inner = inner_ref.as_ref().unwrap();
                (inner.connection.clone(), inner.manager.clone())
            };

            connection
                .update_async_future(
                    resource
                        .print_sparql_update(Some(&manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;

            obj.emit("activities-updated", &[]).unwrap();
            Ok(())
        }

        pub async fn save_weight(
            &self,
            obj: &super::Database,
            weight: Weight,
        ) -> Result<(), glib::Error> {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:WeightMeasurement");
            resource.set_string(
                "health:weight_date",
                &format!("{}", &weight.date.date().format("%Y-%m-%d")),
            );
            resource.set_double(
                "health:weight",
                weight.weight.get::<uom::si::mass::kilogram>().into(),
            );

            let (connection, manager) = {
                let inner_ref = self.inner.borrow();
                let inner = inner_ref.as_ref().unwrap();
                (inner.connection.clone(), inner.manager.clone())
            };

            connection
                .update_async_future(&format!(
                    "DELETE WHERE {{ ?u health:weight_date '{}'^^xsd:date }}; {}",
                    &weight.date.date().format("%Y-%m-%d"),
                    resource
                        .print_sparql_update(Some(&manager), None)
                        .unwrap()
                        .as_str()
                ))
                .await?;

            obj.emit("weights-updated", &[]).unwrap();
            Ok(())
        }
    }
}

glib::wrapper! {
    pub struct Database(ObjectSubclass<imp::Database>);
}

impl Database {
    pub fn new() -> Result<Self, glib::Error> {
        let o = glib::Object::new(&[]).expect("Failed to create Database");

        imp::Database::from_instance(&o).connect(None, None)?;

        Ok(o)
    }

    #[cfg(test)]
    pub fn new_with_store_path(store_path: std::path::PathBuf) -> Result<Self, glib::Error> {
        let o = glib::Object::new(&[]).expect("Failed to create Database");

        let mut path = std::path::PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("data/tracker");

        imp::Database::from_instance(&o).connect(Some(path), Some(store_path))?;

        Ok(o)
    }

    pub fn connect_activities_updated<F: Fn() + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("activities-updated", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub fn connect_weights_updated<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("weights-updated", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    pub async fn get_activities(
        &self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Activity>, glib::Error> {
        imp::Database::from_instance(self)
            .get_activities(date_opt)
            .await
    }

    pub async fn get_steps(&self, date: DateTime<FixedOffset>) -> Result<Vec<Steps>, glib::Error> {
        imp::Database::from_instance(self).get_steps(date).await
    }

    pub async fn get_weights(
        &self,
        date: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Weight>, glib::Error> {
        imp::Database::from_instance(self).get_weights(date).await
    }

    pub async fn get_weight_exists_on_date(
        &self,
        date: DateTime<FixedOffset>,
    ) -> Result<bool, glib::Error> {
        imp::Database::from_instance(self)
            .get_weight_exists_on_date(date)
            .await
    }

    pub async fn import_steps(&self, steps: &[Steps]) -> Result<(), glib::Error> {
        imp::Database::from_instance(self)
            .import_steps(self, steps)
            .await
    }

    pub async fn import_weights(&self, weight: &[Weight]) -> Result<(), glib::Error> {
        imp::Database::from_instance(self)
            .import_weights(self, weight)
            .await
    }

    pub async fn save_activity(&self, activity: Activity) -> Result<(), glib::Error> {
        imp::Database::from_instance(self)
            .save_activity(self, activity)
            .await
    }

    pub async fn save_weight(&self, weight: Weight) -> Result<(), glib::Error> {
        imp::Database::from_instance(self)
            .save_weight(self, weight)
            .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{core::utils::run_async_test_fn, model::ActivityType};
    use chrono::{Duration, Local};
    use tempfile::tempdir;
    use uom::si::{f32::Mass, mass::kilogram};

    #[test]
    fn construct() {
        let data_dir = tempdir().unwrap();
        Database::new_with_store_path(data_dir.path().into()).unwrap();
    }

    #[test]
    fn check_doesnt_exist_activity() {
        let expected_activity = Activity::new();
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

        expected_activity
            .set_activity_type(ActivityType::Walking)
            .set_date(date.into());

        let retrieved_activities = run_async_test_fn(async move {
            db.save_activity(expected_activity).await.unwrap();

            db.get_activities(Some(
                date.checked_add_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        assert!(retrieved_activities.is_empty());
    }

    #[test]
    fn check_doesnt_exists_weight() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = run_async_test_fn(async move {
            db.save_weight(w).await.unwrap();

            db.get_weights(Some(
                date.checked_add_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        assert!(retrieved_weights.is_empty());
    }

    #[test]
    fn check_exists_activity() {
        let expected_activity = Activity::new();
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

        expected_activity
            .set_activity_type(ActivityType::Walking)
            .set_date(date.into())
            .set_steps(Some(50));
        let a = expected_activity.clone();

        let retrieved_activities = run_async_test_fn(async move {
            db.save_activity(a).await.unwrap();

            db.get_activities(Some(
                date.checked_sub_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        let activity = retrieved_activities.get(0).unwrap();
        assert_eq!(
            expected_activity.get_activity_type(),
            activity.get_activity_type()
        );
        assert_eq!(expected_activity.get_steps(), activity.get_steps());
    }

    #[test]
    fn check_exists_weight() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = run_async_test_fn(async move {
            db.save_weight(w).await.unwrap();

            db.get_weights(Some(
                date.checked_sub_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
    }
}
