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

use crate::{
    config,
    model::{Activity, ActivityType, Steps, Weight},
    views::SplitBar,
};
use anyhow::Result;
use chrono::{Date, DateTime, Duration, FixedOffset, NaiveDate, SecondsFormat, Utc};
use gtk::{
    gio::{self, subclass::prelude::*},
    glib::{self, prelude::*},
};
use num_traits::cast::{FromPrimitive, ToPrimitive};
use std::{
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
};
use tracker::prelude::*;
use uom::si::{
    length::{meter, Length},
    mass::{kilogram, Mass},
};

mod imp {
    use gtk::{
        gio::subclass::prelude::*,
        glib::{self, subclass::Signal},
    };
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct DatabaseMut {
        pub connection: tracker::SparqlConnection,
        pub manager: tracker::NamespaceManager,
    }

    #[derive(Debug, Default)]
    pub struct Database {
        pub inner: RefCell<Option<DatabaseMut>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Database {
        const NAME: &'static str = "HealthDatabase";
        type ParentType = glib::Object;
        type Type = super::Database;
    }

    impl ObjectImpl for Database {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("activities-updated", &[], glib::Type::UNIT.into()).build(),
                    Signal::builder("weights-updated", &[], glib::Type::UNIT.into()).build(),
                ]
            });

            SIGNALS.as_ref()
        }
    }
}
static mut DATABASE: Option<Database> = None;

glib::wrapper! {
    /// Helper class to add and retrieve data to and from the Tracker Database.
    pub struct Database(ObjectSubclass<imp::Database>);
}

impl Default for Database {
    fn default() -> Self {
        Self::instance()
    }
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
    pub async fn activities(
        &self,
        date_opt: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Activity>> {
        let self_ = self.imp();

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

            for i in 0..cursor.n_columns() {
                match cursor.variable_name(i).unwrap().as_str() {
                    "id" => {
                        activity
                            .set_activity_type(ActivityType::from_i64(cursor.integer(i)).unwrap());
                    }
                    "date" => {
                        activity.set_date(
                            DateTime::parse_from_rfc3339(cursor.string(i).unwrap().as_str())
                                .unwrap(),
                        );
                    }
                    "calories_burned" => {
                        activity.set_calories_burned(Some(cursor.integer(i).try_into().unwrap()));
                    }
                    "distance" => {
                        activity.set_distance(Some(Length::new::<meter>(cursor.integer(i) as f32)));
                    }
                    "heart_rate_avg" => {
                        activity.set_heart_rate_avg(Some(cursor.integer(i).try_into().unwrap()));
                    }
                    "heart_rate_max" => {
                        activity.set_heart_rate_max(Some(cursor.integer(i).try_into().unwrap()));
                    }
                    "heart_rate_min" => {
                        activity.set_heart_rate_min(Some(cursor.integer(i).try_into().unwrap()));
                    }
                    "minutes" => {
                        activity.set_duration(Duration::minutes(cursor.integer(i)));
                    }
                    "steps" => {
                        activity.set_steps(Some(cursor.integer(i).try_into().unwrap()));
                    }
                    _ => unimplemented!(),
                }
            }

            ret.push(activity);
        }
        //when tracker ordering is fixed, sparql query will order by desc date
        //ret.sort_by_key(crate::Activity::date);

        Ok(ret)
    }

    /// Get calories.
    ///
    /// # Arguments
    /// * `minimum_date` - Only get calorie data (in SplitBar format) that are more recent than `minimum_date`.
    ///
    /// # Returns
    /// An array of [SplitBar]s that are within the given timeframe or a [glib::Error] if querying the DB goes wrong.
    pub async fn calories(&self, minimum_date: DateTime<FixedOffset>) -> Result<Vec<SplitBar>> {
        let connection = {
            self.imp()
                .inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .clone()
        };

        let cursor = connection.query_async_future(&format!("SELECT ?date ?id ?calories_burned WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id ; health:calories_burned ?calories_burned. FILTER  (?date >= '{}'^^xsd:dateTime) }}", minimum_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
        let mut hashmap: std::collections::HashMap<
            DateTime<FixedOffset>,
            std::collections::HashMap<ActivityType, i64>,
        > = std::collections::HashMap::new();

        while let Ok(true) = cursor.next_async_future().await {
            let date = DateTime::parse_from_rfc3339(cursor.string(0).unwrap().as_str()).unwrap();
            let id = ActivityType::from_i64(cursor.integer(1)).unwrap();
            let calories = cursor.integer(2);
            let new_map = |id, calories| {
                let mut hashmap: std::collections::HashMap<ActivityType, i64> =
                    std::collections::HashMap::new();
                hashmap.insert(id, calories);
                hashmap
            };
            hashmap
                .entry(date)
                .or_insert_with(|| new_map(id.clone(), calories));
            if hashmap.contains_key(&date) {
                let calories_before = *hashmap.get(&date).unwrap().get(&id).unwrap_or(&0);
                hashmap
                    .get_mut(&date)
                    .unwrap()
                    .insert(id, calories + calories_before);
            }
        }

        let mut v: Vec<SplitBar> = hashmap
            .drain()
            .map(|(date, bar)| SplitBar {
                date: date.date(),
                calorie_split: bar,
            })
            .collect();

        v.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(v)
    }

    /// Get activities.
    ///
    /// # Arguments
    /// * `minimum_date` - most frequent activities in desc order: on or after a`minimum_date`.
    ///
    /// # Returns
    /// An array of most frequent [ActivityType]s that are within the given timeframe, or a [glib::Error] if querying the DB goes wrong.

    pub async fn most_frequent_activities(
        &self,
        minimum_date: DateTime<FixedOffset>,
    ) -> Result<Vec<ActivityType>> {
        let connection = {
            self.imp()
                .inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .clone()
        };

        let mut most_frequent = Vec::new();

        let cursor = connection.query_async_future(&format!("SELECT ?id WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id ; health:calories_burned ?calories_burned . FILTER  (?date >= '{}'^^xsd:dateTime) }} GROUP BY ?id ORDER BY DESC (SUM(?calories_burned))", minimum_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
        while let Ok(true) = cursor.next_async_future().await {
            most_frequent.push(ActivityType::from_i64(cursor.integer(0)).unwrap());
        }

        Ok(most_frequent)
    }

    pub async fn num_activities(&self) -> Result<i64> {
        let connection = {
            self.imp()
                .inner
                .borrow()
                .as_ref()
                .unwrap()
                .connection
                .clone()
        };
        let cursor = connection
            .query_async_future("SELECT COUNT (?datapoint) WHERE { ?datapoint a health:Activity }")
            .await?;
        cursor.next_async_future().await?;
        Ok(cursor.integer(0))
    }

    #[cfg(test)]
    pub fn connection(&self) -> tracker::SparqlConnection {
        let self_ = self.imp();
        self_.inner.borrow().as_ref().unwrap().connection.clone()
    }

    pub fn instance() -> Self {
        unsafe {
            DATABASE.as_ref().map_or_else(
                || {
                    let database = Self::new().expect("Failed to connect to Tracker Database!");
                    DATABASE = Some(database.clone());
                    database
                },
                std::clone::Clone::clone,
            )
        }
    }

    #[cfg(test)]
    pub fn manager(&self) -> tracker::NamespaceManager {
        let self_ = self.imp();
        self_.inner.borrow().as_ref().unwrap().manager.clone()
    }

    /// Get steps.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get steps that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Steps]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn steps(&self, date: DateTime<FixedOffset>) -> Result<Vec<Steps>> {
        let self_ = self.imp();

        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        let cursor = connection.query_async_future(&format!("SELECT ?date ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:steps ?steps . FILTER  (?date >= '{}'^^xsd:dateTime)}}  ORDER BY ?date", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;
        let mut hashmap = std::collections::HashMap::new();

        while let Ok(true) = cursor.next_async_future().await {
            hashmap.insert(
                DateTime::parse_from_rfc3339(cursor.string(0).unwrap().as_str()).unwrap(),
                hashmap
                    .get(&DateTime::parse_from_rfc3339(cursor.string(0).unwrap().as_str()).unwrap())
                    .unwrap_or(&0)
                    + u32::try_from(cursor.integer(1)).unwrap(),
            );
        }

        let mut v: Vec<Steps> = hashmap
            .drain()
            .map(|(date, steps)| Steps::new(date, steps))
            .collect();

        v.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(v)
    }

    /// Get today's steps.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get steps that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Steps]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn todays_steps(&self, date: DateTime<FixedOffset>) -> Result<i64> {
        let self_ = self.imp();

        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        let cursor = connection.query_async_future(&format!("SELECT SUM(?steps) WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:steps ?steps . FILTER  (?date >= '{}'^^xsd:dateTime)}}", date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))).await?;

        let steps = if let Ok(true) = cursor.next_async_future().await {
            cursor.integer(0)
        } else {
            0
        };

        Ok(steps)
    }

    /// Get weights.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get weights that are more recent than `date_opt`
    ///
    /// # Returns
    /// An array of [Weight]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn weights(&self, date_opt: Option<DateTime<FixedOffset>>) -> Result<Vec<Weight>> {
        let self_ = self.imp();

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
                DateTime::parse_from_rfc3339(cursor.string(0).unwrap().as_str()).unwrap(),
                Mass::new::<kilogram>(cursor.double(1) as f32),
            ));
        }

        // FIXME: The DB should sort this.
        ret.sort_by_key(|a| a.date);

        Ok(ret)
    }

    /// Check if a [Weight] exists on a given date
    ///
    /// # Arguments
    /// * `date` - The date which should be checked
    ///
    /// # Returns
    /// True if a [Weight] exists on the `date`, or [glib::Error] if querying the DB goes wrong.
    pub async fn weight_exists_on_date(&self, date: Date<FixedOffset>) -> Result<bool> {
        let self_ = self.imp();

        let connection = { self_.inner.borrow().as_ref().unwrap().connection.clone() };
        let cursor = connection.query_async_future(&format!("ASK {{ ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date ; health:weight ?weight . FILTER(?date >= '{}'^^xsd:date && ?date < '{}'^^xsd:date) }}", date.format("%Y-%m-%d"), (date + Duration::days(1)).format("%Y-%m-%d"))).await?;

        assert!(cursor.next_async_future().await?);

        Ok(cursor.is_boolean(0))
    }

    /// Import an array of [Steps] into the DB (e.g. when doing the initial sync with a sync provider)
    ///
    /// # Arguments
    /// * `steps` - An array of steps to add to the DB.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn import_steps(&self, steps: &[Steps]) -> Result<()> {
        let self_ = self.imp();

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

        self.emit_by_name("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Import an array of [Weight] into the DB (e.g. when doing the initial sync with a sync provider)
    ///
    /// # Arguments
    /// * `weight` - An array of weight to add to the DB.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn import_weights(&self, weights: &[Weight]) -> Result<()> {
        let self_ = self.imp();

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
                "health:weight_datetime",
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

        self.emit_by_name("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Migrate from an older DB version to a newer one. The migration is one-way (as in you can't switch back to older versions).
    /// This can be called multiple times without problems, the migration just won't do anything afterwards.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn migrate(&self) -> Result<()> {
        self.migrate_activities_date_datetime().await?;
        self.migrate_weight_date_datetime().await?;
        Ok(())
    }

    /// Migrate [Activity]s from `xsd:date` to `xsd:dateTime`. This will set all entries where a date is set to the date at 00:00:00 at the local datetime.
    ///
    /// # Returns
    /// Am error if querying the DB goes wrong.
    pub async fn migrate_activities_date_datetime(&self) -> Result<()> {
        let self_ = self.imp();
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

            for i in 0..cursor.n_columns() {
                match cursor.variable_name(i).unwrap().as_str() {
                    "id" => {
                        resource.set_int64("health:activity_id", cursor.integer(i));
                    }
                    "date" => {
                        resource.set_string(
                            "health:activity_datetime",
                            &DateTime::<Utc>::from_utc(
                                NaiveDate::parse_from_str(
                                    cursor.string(i).unwrap().as_str(),
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
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:calories_burned", v);
                        }
                    }
                    "distance" => {
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:distance", v);
                        }
                    }
                    "heart_rate_avg" => {
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_avg", v);
                        }
                    }
                    "heart_rate_max" => {
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_max", v);
                        }
                    }
                    "heart_rate_min" => {
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:hearth_rate_min", v);
                        }
                    }
                    "minutes" => {
                        let v = cursor.integer(i);
                        if v != 0 {
                            resource.set_int64("health:minutes", v);
                        }
                    }
                    "steps" => {
                        let v = cursor.integer(i);
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

        self.emit_by_name("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Migrate `Activity`s from date to dateTime. This will set all entries where a date is set to the date at 00:00:00 at the local datetime.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn migrate_weight_date_datetime(&self) -> Result<()> {
        let self_ = self.imp();
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
                    NaiveDate::parse_from_str(cursor.string(0).unwrap().as_str(), "%Y-%m-%d")
                        .unwrap()
                        .and_hms(0, 0, 0),
                    Utc,
                )
                .with_timezone(&chrono::Local)
                .to_rfc3339_opts(SecondsFormat::Secs, true),
            );
            resource.set_double("health:weight", cursor.double(1));

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

        self.emit_by_name("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Create a new Tracker DB and connect to Tracker.
    ///
    /// # Returns
    /// Either [Database], or [glib::Error] if connecting to Tracker failed.
    fn new() -> Result<Self> {
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
    pub fn new_with_store_path(store_path: PathBuf) -> Result<Self> {
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
    pub async fn reset(&self) -> Result<()> {
        let self_ = self.imp();
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
    pub async fn save_activity(&self, activity: Activity) -> Result<()> {
        let self_ = self.imp();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_datetime",
            &activity.date().to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        resource.set_int64(
            "health:activity_id",
            activity.activity_type().to_u32().unwrap().into(),
        );

        if let Some(c) = activity.calories_burned() {
            resource.set_int64("health:calories_burned", c.into());
        }
        if let Some(d) = activity.distance() {
            resource.set_int64(
                "health:distance",
                d.get::<uom::si::length::kilometer>() as i64,
            );
        }
        if let Some(avg) = activity.heart_rate_avg() {
            resource.set_int64("health:hearth_rate_avg", avg.into());
        }
        if let Some(max) = activity.heart_rate_max() {
            resource.set_int64("health:hearth_rate_max", max.into());
        }
        if let Some(min) = activity.heart_rate_min() {
            resource.set_int64("health:hearth_rate_min", min.into());
        }
        if activity.duration().num_minutes() != 0 {
            resource.set_int64("health:minutes", activity.duration().num_minutes());
        }
        if let Some(s) = activity.steps() {
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

        self.emit_by_name("activities-updated", &[]).unwrap();
        Ok(())
    }

    /// Save an [Weight] to the database.
    ///
    /// # Arguments
    /// * `weight` - The [Weight] which should be saved.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn save_weight(&self, weight: Weight) -> Result<()> {
        let self_ = self.imp();
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

        self.emit_by_name("weights-updated", &[]).unwrap();
        Ok(())
    }

    /// Connect to the tracker DB. This has to be called before calling any other methods on this struct.
    ///
    /// # Arguments
    /// * `ontology_path` - `Some` if a custom path for the Tracker ontology path is desired (e.g. in tests), or `None` to use the default.
    /// * `store_path` - `Some` if a custom store path for the Tracker DB is desired (e.g. in tests), or `None` to use the default.
    fn connect(&self, ontology_path: Option<PathBuf>, store_path: Option<PathBuf>) -> Result<()> {
        let mut store_path = store_path.unwrap_or_else(glib::user_data_dir);
        store_path.push("health");

        let ontology_path = if config::APPLICATION_ID.ends_with("Devel") {
            let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
            path.push("data");
            path.push("tracker");
            path.push("ontology");
            if path.exists() {
                path
            } else {
                let mut ontology_path = ontology_path
                    .unwrap_or_else(|| Path::new(crate::config::PKGDATADIR).to_path_buf());
                ontology_path.push("ontology");
                ontology_path
            }
        } else {
            let mut ontology_path =
                ontology_path.unwrap_or_else(|| Path::new(crate::config::PKGDATADIR).to_path_buf());
            ontology_path.push("ontology");
            ontology_path
        };

        let manager = tracker::NamespaceManager::new();
        manager.add_prefix("health", "https://gitlab.gnome.org/World/health#");

        self.imp().inner.replace(Some(imp::DatabaseMut {
            connection: tracker::SparqlConnection::new(
                tracker::SparqlConnectionFlags::NONE,
                Some(&gio::File::for_path(store_path)),
                Some(&gio::File::for_path(ontology_path)),
                None::<&gio::Cancellable>,
            )?,
            manager,
        }));

        Ok(())
    }

    fn imp(&self) -> &imp::Database {
        imp::Database::from_instance(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{core::utils::prelude::*, model::ActivityType};
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

        let retrieved_activities = async move {
            db.save_activity(expected_activity).await.unwrap();

            db.activities(Some((date + Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
        assert!(retrieved_activities.is_empty());
    }

    #[test]
    fn check_doesnt_exists_weight() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = async move {
            db.save_weight(w).await.unwrap();

            db.weights(Some((date + Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
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

        let retrieved_activities = async move {
            db.save_activity(a).await.unwrap();

            db.activities(Some((date - Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
        let activity = retrieved_activities.get(0).unwrap();
        assert_eq!(expected_activity.activity_type(), activity.activity_type());
        assert_eq!(expected_activity.steps(), activity.steps());
    }

    #[test]
    fn check_exists_weight() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = async move {
            db.save_weight(w).await.unwrap();

            db.weights(Some((date - Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
    }

    #[test]
    fn migration_activities() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.connection();
        let expected_activity = Activity::new();
        let manager = db.manager();
        let resource = tracker::Resource::new(None);

        expected_activity
            .set_activity_type(ActivityType::Walking)
            .set_date(date.into())
            .set_steps(Some(50));

        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_date",
            &format!("{}", &expected_activity.date().date().format("%Y-%m-%d")),
        );
        resource.set_int64(
            "health:activity_id",
            expected_activity.activity_type().to_u32().unwrap().into(),
        );
        resource.set_int64("health:steps", expected_activity.steps().unwrap().into());

        connection
            .update(
                resource
                    .print_sparql_update(Some(&manager), None)
                    .unwrap()
                    .as_str(),
                None::<&gio::Cancellable>,
            )
            .unwrap();

        let retrieved_activities = async move {
            db.migrate().await.unwrap();
            db.activities(Some((date - Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
        let activity = retrieved_activities.get(0).unwrap();
        assert_eq!(expected_activity.steps(), activity.steps());
        assert_eq!(
            expected_activity
                .date()
                .date()
                .and_hms(0, 0, 0)
                .to_rfc3339(),
            activity.date().with_timezone(&chrono::Utc).to_rfc3339()
        );
        assert_eq!(
            expected_activity.activity_type().to_u32().unwrap(),
            activity.activity_type().to_u32().unwrap()
        );
    }

    #[test]
    fn migration_weights() {
        let data_dir = tempdir().unwrap();
        let date = Local::now();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.connection();
        let expected_weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));
        let manager = db.manager();
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

        let retrieved_weights = async move {
            db.migrate().await.unwrap();
            db.weights(Some((date - Duration::days(1)).into()))
                .await
                .unwrap()
        }
        .block();
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
        assert_eq!(
            expected_weight.date.date().and_hms(0, 0, 0).to_rfc3339(),
            weight.date.with_timezone(&chrono::Utc).to_rfc3339()
        );
    }
}
