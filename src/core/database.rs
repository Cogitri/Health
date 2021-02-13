/* database.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use crate::model::{Activity, ActivityType, Steps, Weight};
use chrono::{Date, DateTime, Duration, FixedOffset, NaiveDate, SecondsFormat, Utc};
use glib::{subclass::types::ObjectSubclass, ObjectExt};
use num_traits::cast::{FromPrimitive, ToPrimitive};
use std::{
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
};
use uom::si::{
    length::{meter, Length},
    mass::{kilogram, Mass},
};

mod imp {
    use glib::subclass::{self, Signal};
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct DatabaseMut {
        pub connection: tracker::SparqlConnection,
        pub manager: tracker::NamespaceManager,
    }

    #[derive(Debug)]
    pub struct Database {
        pub inner: RefCell<Option<DatabaseMut>>,
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
}

glib::wrapper! {
    /// Helper class to add and retrieve data to and from the Tracker Database.
    pub struct Database(ObjectSubclass<imp::Database>);
}

impl Database {
    /// Connect to the `activities-updated` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `activities-update` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
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

    /// Connect to the `weights-updated` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `weights-update` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_weights_updated<F: Fn() + 'static>(&self, callback: F) -> glib::SignalHandlerId {
        self.connect_local("weights-updated", false, move |_| {
            callback();
            None
        })
        .unwrap()
    }

    /// Get activities.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get activities that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Activity]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn get_activities(
        &self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Activity>, glib::Error> {
        let self_ = self.get_priv();

        let cursor = if let Some(date) = date_opt {
            let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
            connection.query_async_future(&format!("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id . OPTIONAL {{ ?datapoint health:calories_burned ?calories_burned . }} OPTIONAL {{ ?datapoint health:distance ?distance . }} OPTIONAL {{ ?datapoint health:hearth_rate_avg ?heart_rate_avg . }} OPTIONAL {{ ?datapoint health:hearth_rate_min ?heart_rate_min . }} OPTIONAL {{ ?datapoint health:hearth_rate_max ?heart_rate_max . }} OPTIONAL {{ ?datapoint health:steps ?steps . }} OPTIONAL {{ ?datapoint health:minutes ?minutes }} FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY DESC(?date)", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
        } else {
            let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
            connection.query_async_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?
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
                        activity.set_date(
                            DateTime::parse_from_rfc3339(cursor.get_string(i).0.unwrap().as_str())
                                .unwrap(),
                        );
                    }
                    "calories_burned" => {
                        activity
                            .set_calories_burned(Some(cursor.get_integer(i).try_into().unwrap()));
                    }
                    "distance" => {
                        activity
                            .set_distance(Some(Length::new::<meter>(cursor.get_integer(i) as f32)));
                    }
                    "heart_rate_avg" => {
                        activity
                            .set_heart_rate_avg(Some(cursor.get_integer(i).try_into().unwrap()));
                    }
                    "heart_rate_max" => {
                        activity
                            .set_heart_rate_max(Some(cursor.get_integer(i).try_into().unwrap()));
                    }
                    "heart_rate_min" => {
                        activity
                            .set_heart_rate_min(Some(cursor.get_integer(i).try_into().unwrap()));
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

    #[cfg(test)]
    pub fn get_connection(&self) -> tracker::SparqlConnection {
        let self_ = self.get_priv();
        self_.inner.borrow().as_ref().unwrap().connection.clone()
    }

    #[cfg(test)]
    pub fn get_manager(&self) -> tracker::NamespaceManager {
        let self_ = self.get_priv();
        self_.inner.borrow().as_ref().unwrap().manager.clone()
    }

    /// Get steps.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get steps that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Steps]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn get_steps(&self, date: DateTime<FixedOffset>) -> Result<Vec<Steps>, glib::Error> {
        let self_ = self.get_priv();

        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        let cursor = connection.query_async_future(&format!("SELECT ?date ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:steps ?steps . FILTER  (?date >= '{}'^^xsd:dateTime)}}", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
        let mut hashmap = std::collections::HashMap::new();

        while let Ok(true) = cursor.next_async_future().await {
            hashmap.insert(
                DateTime::parse_from_rfc3339(cursor.get_string(0).0.unwrap().as_str()).unwrap(),
                hashmap.get(&date).unwrap_or(&0) + u32::try_from(cursor.get_integer(1)).unwrap(),
            );
        }

        let mut v: Vec<Steps> = hashmap
            .drain()
            .map(|(date, steps)| Steps::new(date, steps))
            .collect();

        v.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(v)
    }

    /// Get weights.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get weights that are more recent than `date_opt`
    ///
    /// # Returns
    /// An array of [Weight]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn get_weights(
        &self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Weight>, glib::Error> {
        let self_ = self.get_priv();

        let cursor = if let Some(date) = date_opt {
            let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
            connection.query_async_future(&format!("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date  ; health:weight ?weight . FILTER  (?date >= '{}'^^xsd:dateTime)}} ORDER BY ?date", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?
        } else {
            let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
            connection.query_async_future("SELECT ?date ?weight WHERE { ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date  ; health:weight ?weight . } ORDER BY ?date").await?
        };
        let mut ret = Vec::new();

        while let Ok(true) = cursor.next_async_future().await {
            ret.push(Weight::new(
                DateTime::parse_from_rfc3339(cursor.get_string(0).0.unwrap().as_str()).unwrap(),
                Mass::new::<kilogram>(cursor.get_double(1) as f32),
            ));
        }

        Ok(ret)
    }

    /// Check if a [Weight] exists on a given date
    ///
    /// # Arguments
    /// * `date` - The date which should be checked
    ///
    /// # Returns
    /// True if a [Weight] exists on the `date`, or [glib::Error] if querying the DB goes wrong.
    pub async fn get_weight_exists_on_date(
        &self,
        date: Date<FixedOffset>,
    ) -> Result<bool, glib::Error> {
        let self_ = self.get_priv();

        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        let cursor = connection.query_async_future(&format!("ASK {{ ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date ; health:weight ?weight . FILTER(?date >= '{}'^^xsd:date && ?date < '{}'^^xsd:date) }}", date.format("%Y-%m-%d"), (date + Duration::days(1)).format("%Y-%m-%d"))).await?;

        assert!(cursor.next_async_future().await?);

        return Ok(cursor.get_boolean(0));
    }

    /// Import an array of [Steps] into the DB (e.g. when doing the initial sync with a sync provider)
    ///
    /// # Arguments
    /// * `steps` - An array of steps to add to the DB.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn import_steps(&self, steps: &[Steps]) -> Result<(), glib::Error> {
        let self_ = self.get_priv();

        if steps.is_empty() {
            return Ok(());
        }

        let (connection, manager) = {
            let inner_ref = self_.inner.borrow();
            let inner = inner_ref.as_ref().unwrap();
            (inner.connection.clone(), inner.manager.clone())
        };

        for s in steps {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:Activity");
            resource.set_string(
                "health:activity_datetime",
                &s.date.to_rfc3339_opts(SecondsFormat::Secs, true),
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

        self.emit("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Import an array of [Weight] into the DB (e.g. when doing the initial sync with a sync provider)
    ///
    /// # Arguments
    /// * `weight` - An array of weight to add to the DB.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn import_weights(&self, weights: &[Weight]) -> Result<(), glib::Error> {
        let self_ = self.get_priv();

        if weights.is_empty() {
            return Ok(());
        }

        let (connection, manager) = {
            let inner_ref = self_.inner.borrow();
            let inner = inner_ref.as_ref().unwrap();
            (inner.connection.clone(), inner.manager.clone())
        };

        for w in weights {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:WeightMeasurement");
            resource.set_string(
                "health:weight_date",
                &w.date.to_rfc3339_opts(SecondsFormat::Secs, true),
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

        self.emit("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Migrate from an older DB version to a newer one. The migration is one-way (as in you can't switch back to older versions).
    /// This can be called multiple times without problems, the migration just won't do anything afterwards.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn migrate(&self) -> Result<(), glib::Error> {
        self.migrate_activities_date_datetime().await?;
        self.migrate_weight_date_datetime().await?;
        Ok(())
    }

    /// Migrate [Activity]s from `xsd:date` to `xsd:dateTime`. This will set all entries where a date is set to the date at 00:00:00 at the local datetime.
    ///
    /// # Returns
    /// Am error if querying the DB goes wrong.
    pub async fn migrate_activities_date_datetime(&self) -> Result<(), glib::Error> {
        let self_ = self.get_priv();
        let (connection, manager) = {
            let inner_ref = self_.inner.borrow();
            let inner = inner_ref.as_ref().unwrap();
            (inner.connection.clone(), inner.manager.clone())
        };

        let cursor =
        connection.query_async_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?;

        while let Ok(true) = cursor.next_async_future().await {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:Activity");

            for i in 0..cursor.get_n_columns() {
                match cursor.get_variable_name(i).unwrap().as_str() {
                    "id" => {
                        resource.set_int64("health:activity_id", cursor.get_integer(i));
                    }
                    "date" => {
                        resource.set_string(
                            "health:activity_datetime",
                            &DateTime::<Utc>::from_utc(
                                NaiveDate::parse_from_str(
                                    cursor.get_string(i).0.unwrap().as_str(),
                                    "%Y-%m-%d",
                                )
                                .unwrap()
                                .and_hms(0, 0, 0),
                                Utc,
                            )
                            .with_timezone(&chrono::Local)
                            .to_rfc3339_opts(SecondsFormat::Secs, true),
                        );
                    }
                    "calories_burned" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:calories_burned", v);
                        }
                    }
                    "distance" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:distance", v);
                        }
                    }
                    "heart_rate_avg" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_avg", v);
                        }
                    }
                    "heart_rate_max" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_max", v);
                        }
                    }
                    "heart_rate_min" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_min", v);
                        }
                    }
                    "minutes" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:minutes", v);
                        }
                    }
                    "steps" => {
                        let v = cursor.get_integer(i);
                        if v != 0 {
                            resource.set_int64("health:steps", v);
                        }
                    }
                    _ => unimplemented!(),
                }
            }

            connection
                .update_async_future(
                    resource
                        .print_sparql_update(Some(&manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        connection
            .update_async_future(
                "DELETE WHERE { ?datapoint a health:Activity; health:activity_date ?date };",
            )
            .await?;

        self.emit("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Migrate `Activity`s from date to dateTime. This will set all entries where a date is set to the date at 00:00:00 at the local datetime.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn migrate_weight_date_datetime(&self) -> Result<(), glib::Error> {
        let self_ = self.get_priv();
        let (connection, manager) = {
            let inner_ref = self_.inner.borrow();
            let inner = inner_ref.as_ref().unwrap();
            (inner.connection.clone(), inner.manager.clone())
        };

        let cursor =
        connection.query_async_future("SELECT ?date ?weight WHERE { ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . } ORDER BY ?date").await?;

        while let Ok(true) = cursor.next_async_future().await {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:WeightMeasurement");
            resource.set_string(
                "health:weight_datetime",
                &DateTime::<Utc>::from_utc(
                    NaiveDate::parse_from_str(cursor.get_string(0).0.unwrap().as_str(), "%Y-%m-%d")
                        .unwrap()
                        .and_hms(0, 0, 0),
                    Utc,
                )
                .with_timezone(&chrono::Local)
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            );
            resource.set_double("health:weight", cursor.get_double(1));

            connection
                .update_async_future(
                    resource
                        .print_sparql_update(Some(&manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        connection
            .update_async_future(
                "DELETE WHERE { ?datapoint a health:WeightMeasurement; health:weight_date ?date };",
            )
            .await?;

        self.emit("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Create a new Tracker DB and connect to Tracker.
    ///
    /// # Returns
    /// Either [Database], or [glib::Error] if connecting to Tracker failed.
    pub fn new() -> Result<Self, glib::Error> {
        let o: Self = glib::Object::new(&[]).expect("Failed to create Database");

        o.connect(None, None)?;

        Ok(o)
    }

    /// Create a new Tracker DB and connect to Tracker.
    ///
    /// # Arguments
    /// * `store_path` - [PathBuf] to where the Tracker DB should be stored.
    ///
    /// # Returns
    /// Either [Database], or [glib::Error] if connecting to Tracker failed.
    #[cfg(test)]
    pub fn new_with_store_path(store_path: PathBuf) -> Result<Self, glib::Error> {
        let o: Self = glib::Object::new(&[]).expect("Failed to create Database");

        let mut path = PathBuf::new();
        path.push(env!("CARGO_MANIFEST_DIR"));
        path.push("data/tracker");

        o.connect(Some(path), Some(store_path))?;

        Ok(o)
    }

    /// Reset the DB (as in delete all entries in it).
    ///
    /// # Returns
    /// Returns an error if querying the DB goes wrong.
    pub async fn reset(&self) -> Result<(), glib::Error> {
        let self_ = self.get_priv();
        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        connection
            .update_async_future("DELETE WHERE { ?datapoint a health:WeightMeasurement }")
            .await?;
        connection
            .update_async_future("DELETE WHERE { ?datapoint a health:Activity }")
            .await?;

        Ok(())
    }

    /// Save an [Activity] to the database.
    ///
    /// # Arguments
    /// * `activity` - The [Activity] which should be saved.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn save_activity(&self, activity: Activity) -> Result<(), glib::Error> {
        let self_ = self.get_priv();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_datetime",
            &activity
                .get_date()
                .to_rfc3339_opts(SecondsFormat::Secs, true),
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
            let inner_ref = self_.inner.borrow();
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

        self.emit("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Save an [Weight] to the database.
    ///
    /// # Arguments
    /// * `weight` - The [Weight] which should be saved.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn save_weight(&self, weight: Weight) -> Result<(), glib::Error> {
        let self_ = self.get_priv();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:WeightMeasurement");
        resource.set_string(
            "health:weight_datetime",
            &weight.date.to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        resource.set_double(
            "health:weight",
            weight.weight.get::<uom::si::mass::kilogram>().into(),
        );

        let (connection, manager) = {
            let inner_ref = self_.inner.borrow();
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

        self.emit("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Connect to the tracker DB. This has to be called before calling any other methods on this struct.
    ///
    /// # Arguments
    /// * `ontology_path` - `Some` if a custom path for the Tracker ontology path is desired (e.g. in tests), or `None` to use the default.
    /// * `store_path` - `Some` if a custom store path for the Tracker DB is desired (e.g. in tests), or `None` to use the default.
    fn connect(
        &self,
        ontology_path: Option<PathBuf>,
        store_path: Option<PathBuf>,
    ) -> Result<(), glib::Error> {
        let mut store_path = store_path.unwrap_or_else(glib::get_user_data_dir);
        store_path.push("health");

        let mut ontology_path =
            ontology_path.unwrap_or_else(|| Path::new(crate::config::PKGDATADIR).to_path_buf());
        ontology_path.push("ontology");

        let manager = tracker::NamespaceManager::new();
        manager.add_prefix("health", "https://gitlab.gnome.org/World/health#");

        self.get_priv().inner.replace(Some(imp::DatabaseMut {
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

    fn get_priv(&self) -> &imp::Database {
        imp::Database::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{core::utils::run_async_test_fn, model::ActivityType};
    use chrono::{Duration, Local};
    use num_traits::cast::ToPrimitive;
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

    #[test]
    fn migration_activities() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.get_connection();
        let expected_activity = Activity::new();
        let manager = db.get_manager();
        let resource = tracker::Resource::new(None);

        expected_activity
            .set_activity_type(ActivityType::Walking)
            .set_date(date.into())
            .set_steps(Some(50));

        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_date",
            &format!(
                "{}",
                &expected_activity.get_date().date().format("%Y-%m-%d")
            ),
        );
        resource.set_int64(
            "health:activity_id",
            expected_activity
                .get_activity_type()
                .to_u32()
                .unwrap()
                .into(),
        );
        resource.set_int64(
            "health:steps",
            expected_activity.get_steps().unwrap().into(),
        );

        connection
            .update(
                resource
                    .print_sparql_update(Some(&manager), None)
                    .unwrap()
                    .as_str(),
                None::<&gio::Cancellable>,
            )
            .unwrap();

        let retrieved_activities = run_async_test_fn(async move {
            db.migrate().await.unwrap();
            db.get_activities(Some(
                date.checked_sub_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        let activity = retrieved_activities.get(0).unwrap();
        assert_eq!(expected_activity.get_steps(), activity.get_steps());
        assert_eq!(
            expected_activity
                .get_date()
                .date()
                .and_hms(0, 0, 0)
                .to_rfc3339(),
            activity.get_date().with_timezone(&chrono::Utc).to_rfc3339()
        );
        assert_eq!(
            expected_activity.get_activity_type().to_u32().unwrap(),
            activity.get_activity_type().to_u32().unwrap()
        );
    }

    #[test]
    fn migration_weights() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.get_connection();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let manager = db.get_manager();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:WeightMeasurement");
        resource.set_string(
            "health:weight_date",
            &format!("{}", &expected_weight.date.date().format("%Y-%m-%d")),
        );
        resource.set_double(
            "health:weight",
            expected_weight
                .weight
                .get::<uom::si::mass::kilogram>()
                .into(),
        );

        connection
            .update(
                resource
                    .print_sparql_update(Some(&manager), None)
                    .unwrap()
                    .as_str(),
                None::<&gio::Cancellable>,
            )
            .unwrap();

        let retrieved_weights = run_async_test_fn(async move {
            db.migrate().await.unwrap();
            db.get_weights(Some(
                date.checked_sub_signed(Duration::days(1)).unwrap().into(),
            ))
            .await
            .unwrap()
        });
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
        assert_eq!(
            expected_weight.date.date().and_hms(0, 0, 0).to_rfc3339(),
            weight.date.with_timezone(&chrono::Utc).to_rfc3339()
        );
    }
}
