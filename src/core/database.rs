use std::path::Path;

use crate::model::{Activity, ActivityType, Steps, Weight};
use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use gdk::subclass::prelude::ObjectSubclass;
use glib::ObjectExt;
use num_traits::cast::{FromPrimitive, ToPrimitive};
use uom::si::{
    f32::{Length, Mass},
    mass::kilogram,
};

mod imp {
    use super::*;
    use glib::subclass::{self, Signal};
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use uom::si::length::meter;

    #[derive(Debug)]
    pub struct HealthDatabaseMut {
        connection: tracker::SparqlConnection,
        manager: tracker::NamespaceManager,
    }

    #[derive(Debug)]
    pub struct HealthDatabase {
        inner: RefCell<Option<HealthDatabaseMut>>,
    }

    impl ObjectSubclass for HealthDatabase {
        const NAME: &'static str = "HealthHealthDatabase";
        type ParentType = glib::Object;
        type Instance = subclass::simple::InstanceStruct<Self>;
        type Class = subclass::simple::ClassStruct<Self>;
        type Type = super::HealthDatabase;
        type Interfaces = ();

        glib::object_subclass!();

        fn new() -> Self {
            Self {
                inner: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for HealthDatabase {
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

    impl HealthDatabase {
        pub fn connect(&self) -> Result<(), glib::Error> {
            let mut store_path = glib::get_user_data_dir();
            store_path.push("health");

            let mut ontology_path = Path::new(crate::config::PKGDATADIR).to_path_buf();
            ontology_path.push("ontology");

            let manager = tracker::NamespaceManager::new();
            manager.add_prefix("health", "https://gitlab.gnome.org/World/health#");

            self.inner.replace(Some(HealthDatabaseMut {
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
                self.inner.borrow().as_ref().unwrap().connection.query_async_future(&format!("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL {{ ?datapoint health:calories_burned ?calories_burned . }} OPTIONAL {{ ?datapoint health:distance ?distance . }} OPTIONAL {{ ?datapoint health:hearth_rate_avg ?heart_rate_avg . }} OPTIONAL {{ ?datapoint health:hearth_rate_min ?heart_rate_min . }} OPTIONAL {{ ?datapoint health:hearth_rate_max ?heart_rate_max . }} OPTIONAL {{ ?datapoint health:steps ?steps . }} OPTIONAL {{ ?datapoint health:minutes ?minutes }} FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY DESC(?date)", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
            } else {
                self.inner.borrow().as_ref().unwrap().connection.query_async_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?
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
                            activity.set_calories_burned(Some(cursor.get_integer(i) as u32));
                        }
                        "distance" => {
                            activity.set_distance(Some(Length::new::<meter>(
                                cursor.get_integer(i) as f32,
                            )));
                        }
                        "heart_rate_avg" => {
                            activity.set_heart_rate_avg(Some(cursor.get_integer(i) as u32));
                        }
                        "heart_rate_max" => {
                            activity.set_heart_rate_max(Some(cursor.get_integer(i) as u32));
                        }
                        "heart_rate_min" => {
                            activity.set_heart_rate_min(Some(cursor.get_integer(i) as u32));
                        }
                        "minutes" => {
                            activity.set_duration(Duration::minutes(cursor.get_integer(i)));
                        }
                        "steps" => {
                            activity.set_steps(Some(cursor.get_integer(i) as u32));
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
            let cursor = self.inner.borrow().as_ref().unwrap().connection.query_async_future(&format!("SELECT ?date ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_date ?date ; health:steps ?steps . FILTER  (?date >= '{}'^^xsd:dateTime)}}", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
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
                    hashmap.get(&date).unwrap_or(&0) + cursor.get_integer(1) as u32,
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
                self.inner.borrow().as_ref().unwrap().connection.query_async_future(&format!("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY ?date", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
            } else {
                self.inner.borrow().as_ref().unwrap().connection.query_async_future("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . }} ORDER BY ?date").await?
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
            let cursor = self.inner.borrow().as_ref().unwrap().connection.query_async_future(&format!("ASK {{ ?datapoint a health:WeightMeasurement ; health:weight_date '{}'^^xsd:date; health:weight ?weight . }}", date.date().format("%Y-%m-%d"))).await?;

            assert!(cursor.next_async_future().await?);

            return Ok(cursor.get_boolean(0));
        }

        pub async fn reset(&self) -> Result<(), glib::Error> {
            self.inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .update_async_future("DELETE WHERE { ?datapoint a health:WeightMeasurement }")
                .await?;
            self.inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .update_async_future("DELETE WHERE { ?datapoint a health:Activity }")
                .await?;

            Ok(())
        }

        pub async fn save_activity(
            &self,
            obj: &super::HealthDatabase,
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

            self.inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .update_async_future(
                    resource
                        .print_sparql_update(
                            Some(&self.inner.borrow().as_ref().unwrap().manager),
                            None,
                        )
                        .unwrap()
                        .as_str(),
                )
                .await?;

            obj.emit("activities-updated", &[]).unwrap();
            Ok(())
        }

        pub async fn save_weight(
            &self,
            obj: &super::HealthDatabase,
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

            self.inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .update_async_future(&format!(
                    "DELETE WHERE {{ ?u health:weight_date '{}'^^xsd:date }}; {}",
                    &weight.date.date().format("%Y-%m-%d"),
                    resource
                        .print_sparql_update(
                            Some(&self.inner.borrow().as_ref().unwrap().manager),
                            None
                        )
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
    pub struct HealthDatabase(ObjectSubclass<imp::HealthDatabase>);
}

impl HealthDatabase {
    pub fn new() -> Result<Self, glib::Error> {
        let o = glib::Object::new(&[]).expect("Failed to create HealthDatabase");

        imp::HealthDatabase::from_instance(&o).connect()?;

        return Ok(o);
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
        imp::HealthDatabase::from_instance(self)
            .get_activities(date_opt)
            .await
    }

    pub async fn get_steps(&self, date: DateTime<FixedOffset>) -> Result<Vec<Steps>, glib::Error> {
        imp::HealthDatabase::from_instance(self)
            .get_steps(date)
            .await
    }

    pub async fn get_weights(
        &self,
        date: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Weight>, glib::Error> {
        imp::HealthDatabase::from_instance(self)
            .get_weights(date)
            .await
    }

    pub async fn get_weight_exists_on_date(
        &self,
        date: DateTime<FixedOffset>,
    ) -> Result<bool, glib::Error> {
        imp::HealthDatabase::from_instance(self)
            .get_weight_exists_on_date(date)
            .await
    }

    pub async fn save_activity(&self, activity: Activity) -> Result<(), glib::Error> {
        imp::HealthDatabase::from_instance(self)
            .save_activity(self, activity)
            .await
    }

    pub async fn save_weight(&self, weight: Weight) -> Result<(), glib::Error> {
        imp::HealthDatabase::from_instance(self)
            .save_weight(self, weight)
            .await
    }
}
