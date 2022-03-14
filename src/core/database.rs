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
    model::{Activity, ActivityType, Steps, Weight},
    prelude::*,
    views::SplitBar,
};
use anyhow::Result;
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
    use once_cell::unsync::OnceCell;

    #[derive(Debug, Default)]
    pub struct Database {
        pub connection: OnceCell<tracker::SparqlConnection>,
        pub manager: OnceCell<tracker::NamespaceManager>,
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
    pub fn connect_activities_updated<F: Fn(&Self) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("activities-updated", false, move |values| {
            callback(&values[0].get().unwrap());
            None
        })
    }

    /// Connect to the `weights-updated` signal.
    ///
    /// # Arguments
    /// * `callback` - The callback which should be invoked when `weights-update` is emitted.
    ///
    /// # Returns
    /// A [glib::SignalHandlerId] that can be used for disconnecting the signal if so desired.
    pub fn connect_weights_updated<F: Fn(&Self) + 'static>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("weights-updated", false, move |values| {
            callback(&values[0].get().unwrap());
            None
        })
    }

    /// Get activities.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get activities that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Activity]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn activities(&self, date_opt: Option<glib::DateTime>) -> Result<Vec<Activity>> {
        let imp = self.imp();

        let connection = imp.connection.get().unwrap();
        let cursor = if let Some(date) = date_opt {
            let statement = connection.query_statement("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id . OPTIONAL {{ ?datapoint health:calories_burned ?calories_burned . }} OPTIONAL {{ ?datapoint health:distance ?distance . }} OPTIONAL {{ ?datapoint health:hearth_rate_avg ?heart_rate_avg . }} OPTIONAL {{ ?datapoint health:hearth_rate_min ?heart_rate_min . }} OPTIONAL {{ ?datapoint health:hearth_rate_max ?heart_rate_max . }} OPTIONAL {{ ?datapoint health:steps ?steps . }} OPTIONAL {{ ?datapoint health:minutes ?minutes }} FILTER  (?date >= ~date^^xsd:dateTime)}} ORDER BY DESC(?date)", None::<&gio::Cancellable>).unwrap().unwrap();
            statement.bind_string("date", &date.format_iso8601().unwrap().as_str());
            statement.execute_future().await?
        } else {
            connection.query_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?
        };

        let mut ret = Vec::new();
        while let Ok(true) = cursor.next_future().await {
            let mut activity = Activity::builder();

            for i in 0..cursor.n_columns() {
                match cursor.variable_name(i).unwrap().as_str() {
                    "id" => {
                        activity.activity_type(ActivityType::from_i64(cursor.integer(i)).unwrap());
                    }
                    "date" => {
                        activity.date(glib::DateTime::from_iso8601(
                            cursor.string(i).unwrap().as_str(),
                            None,
                        )?);
                    }
                    "calories_burned" => {
                        activity.calories_burned(cursor.integer(i).try_into().unwrap());
                    }
                    "distance" => {
                        activity.distance(Length::new::<meter>(cursor.integer(i) as f32));
                    }
                    "heart_rate_avg" => {
                        activity.heart_rate_avg(cursor.integer(i).try_into().unwrap());
                    }
                    "heart_rate_max" => {
                        activity.heart_rate_max(cursor.integer(i).try_into().unwrap());
                    }
                    "heart_rate_min" => {
                        activity.heart_rate_min(cursor.integer(i).try_into().unwrap());
                    }
                    "minutes" => {
                        activity.duration(glib::TimeSpan::from_minutes(cursor.integer(i)));
                    }
                    "steps" => {
                        activity.steps(cursor.integer(i).try_into().unwrap());
                    }
                    _ => {
                        glib::g_error!(
                            crate::config::APPLICATION_ID,
                            "Unknown variable name {}",
                            cursor.variable_name(i).unwrap()
                        );
                        unimplemented!();
                    }
                }
            }

            ret.push(activity.build());
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
    pub async fn calories(&self, minimum_date: glib::DateTime) -> Result<Vec<SplitBar>> {
        let connection = self.imp().connection.get().unwrap();
        let statement = connection.query_statement("SELECT ?date ?id ?calories_burned WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id ; health:calories_burned ?calories_burned. FILTER  (?date >= ~date^^xsd:dateTime) }}", None::<&gio::Cancellable>).unwrap().unwrap();
        statement.bind_string("date", &minimum_date.format_iso8601().unwrap().as_str());
        let cursor = statement.execute_future().await?;

        let mut hashmap: std::collections::HashMap<
            glib::DateTime,
            std::collections::HashMap<ActivityType, i64>,
        > = std::collections::HashMap::new();

        while let Ok(true) = cursor.next_future().await {
            let date =
                glib::DateTime::from_iso8601(cursor.string(0).unwrap().as_str(), None).unwrap();
            let id = ActivityType::from_i64(cursor.integer(1)).unwrap();
            let calories = cursor.integer(2);
            let new_map = |id, calories| {
                let mut hashmap: std::collections::HashMap<ActivityType, i64> =
                    std::collections::HashMap::new();
                hashmap.insert(id, calories);
                hashmap
            };
            if hashmap.contains_key(&date) {
                let calories_before = *hashmap.get(&date).unwrap().get(&id).unwrap_or(&0);
                hashmap
                    .get_mut(&date)
                    .unwrap()
                    .insert(id, calories + calories_before);
            }
            hashmap.entry(date).or_insert_with(|| new_map(id, calories));
        }

        let mut v: Vec<SplitBar> = hashmap
            .drain()
            .map(|(date, bar)| SplitBar {
                date,
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
        minimum_date: glib::DateTime,
    ) -> Result<Vec<ActivityType>> {
        let connection = self.imp().connection.get().unwrap();
        let mut most_frequent = Vec::new();

        let statement = connection.query_statement("SELECT ?id WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:activity_id ?id ; health:calories_burned ?calories_burned . FILTER  (?date >= ~date^^xsd:dateTime) }} GROUP BY ?id ORDER BY DESC (SUM(?calories_burned))", None::<&gio::Cancellable>).unwrap().unwrap();
        statement.bind_string("date", &minimum_date.format_iso8601().unwrap().as_str());
        let cursor = statement.execute_future().await?;

        while let Ok(true) = cursor.next_future().await {
            most_frequent.push(ActivityType::from_i64(cursor.integer(0)).unwrap());
        }

        Ok(most_frequent)
    }

    pub async fn has_activities(&self) -> Result<bool> {
        let connection = self.imp().connection.get().unwrap();
        let cursor = connection
            .query_future("ASK { ?datapoint a health:Activity }")
            .await?;
        cursor.next_future().await?;
        Ok(cursor.is_boolean(0))
    }

    #[cfg(test)]
    pub fn connection(&self) -> tracker::SparqlConnection {
        self.imp().connection.get().unwrap().clone()
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
    pub fn set_instance(db: Database) {
        unsafe {
            DATABASE = Some(db);
        }
    }

    #[cfg(test)]
    pub fn manager(&self) -> tracker::NamespaceManager {
        self.imp().manager.get().unwrap().clone()
    }

    /// Get steps.
    ///
    /// # Arguments
    /// * `date_opt` - If `Some`, only get steps that are more recent than `date_opt`.
    ///
    /// # Returns
    /// An array of [Steps]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn steps(&self, date: glib::DateTime) -> Result<Vec<Steps>> {
        let imp = self.imp();

        let connection = imp.connection.get().unwrap();

        let statement = connection.query_statement("SELECT ?date ?steps WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:steps ?steps . FILTER  (?date >= ~date^^xsd:dateTime)}}  ORDER BY ?date", None::<&gio::Cancellable>).unwrap().unwrap();
        statement.bind_string("date", &date.format_iso8601().unwrap());
        let cursor = statement.execute_future().await?;
        let mut hashmap = std::collections::HashMap::new();

        while let Ok(true) = cursor.next_future().await {
            hashmap.insert(
                glib::DateTime::from_iso8601(cursor.string(0).unwrap().as_str(), None).unwrap(),
                hashmap
                    .get(
                        &glib::DateTime::from_iso8601(cursor.string(0).unwrap().as_str(), None)
                            .unwrap(),
                    )
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
    /// # Returns
    /// An array of [Steps]s that are within the given timeframe (if set), or a [glib::Error] if querying the DB goes wrong.
    pub async fn todays_steps(&self) -> Result<i64> {
        let imp = self.imp();
        let date = glib::DateTime::today();

        let connection = imp.connection.get().unwrap();
        let statement = connection.query_statement("SELECT SUM(?steps) WHERE {{ ?datapoint a health:Activity ; health:activity_datetime ?date ; health:steps ?steps . FILTER  (?date >= ~date^^xsd:dateTime)}}", None::<&gio::Cancellable>).unwrap().unwrap();
        statement.bind_string("date", &date.format_iso8601().unwrap());
        let cursor = statement.execute_future().await?;

        let steps = if let Ok(true) = cursor.next_future().await {
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
    pub async fn weights(&self, date_opt: Option<glib::DateTime>) -> Result<Vec<Weight>> {
        let imp = self.imp();

        let connection = imp.connection.get().unwrap();
        let cursor = if let Some(date) = date_opt {
            let statement = connection.query_statement("SELECT ?date ?weight WHERE {{ ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date  ; health:weight ?weight . FILTER  (?date >= ~date^^xsd:dateTime)}} ORDER BY ?date", None::<&gio::Cancellable>).unwrap().unwrap();
            statement.bind_string("date", &date.format_iso8601().unwrap().as_str());
            statement.execute_future().await?
        } else {
            connection.query_future("SELECT ?date ?weight WHERE { ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date  ; health:weight ?weight . } ORDER BY ?date").await?
        };
        let mut ret = Vec::new();

        while let Ok(true) = cursor.next_future().await {
            ret.push(Weight::new(
                glib::DateTime::from_iso8601(cursor.string(0).unwrap().as_str(), None).unwrap(),
                Mass::new::<kilogram>(cursor.double(1) as f32),
            ));
        }

        // FIXME: The DB should sort this.
        ret.sort_by_key(|a| a.date.clone());

        Ok(ret)
    }

    /// Check if a [Weight] exists on a given date
    ///
    /// # Arguments
    /// * `date` - The date which should be checked
    ///
    /// # Returns
    /// True if a [Weight] exists on the `date`, or [glib::Error] if querying the DB goes wrong.
    pub async fn weight_exists_on_date(&self, date: glib::DateTime) -> Result<bool> {
        let imp = self.imp();

        let connection = imp.connection.get().unwrap();
        let statement = connection.query_statement("ASK {{ ?datapoint a health:WeightMeasurement ; health:weight_datetime ?date ; health:weight ?weight . FILTER(?date >= ~date^^xsd:dateTime && ?date < ~nextdate^^xsd:dateTime) }}", None::<&gio::Cancellable>).unwrap().unwrap();
        statement.bind_string("date", &date.reset_hms().format_iso8601().unwrap().as_str());
        statement.bind_string(
            "nextdate",
            &date
                .add_days(1)
                .unwrap()
                .reset_hms()
                .format_iso8601()
                .unwrap()
                .as_str(),
        );
        let cursor = statement.execute_future().await?;

        assert!(cursor.next_future().await?);

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
        let imp = self.imp();

        if steps.is_empty() {
            return Ok(());
        }

        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        for s in steps {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:Activity");
            resource.set_string(
                "health:activity_datetime",
                &s.date.format_iso8601().unwrap().as_str(),
            );
            resource.set_int64("health:steps", s.steps.into());
            resource.set_int64(
                "health:activity_id",
                ActivityType::Walking.to_i64().unwrap(),
            );
            // FIXME: Set correct minutes here
            resource.set_int64("health:minutes", 0);

            connection
                .update_future(
                    resource
                        .print_sparql_update(Some(manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        self.emit_by_name::<()>("activities-updated", &[]);
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
        let imp = self.imp();

        if weights.is_empty() {
            return Ok(());
        }

        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        for w in weights {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:WeightMeasurement");
            resource.set_string(
                "health:weight_datetime",
                &w.date.format_iso8601().unwrap().as_str(),
            );
            resource.set_double("health:weight", w.weight.get::<kilogram>().into());

            connection
                .update_future(
                    resource
                        .print_sparql_update(Some(manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        self.emit_by_name::<()>("weights-updated", &[]);
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
        let imp = self.imp();
        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        let cursor =
        connection.query_future("SELECT ?date ?id ?calories_burned ?distance ?heart_rate_avg ?heart_rate_max ?heart_rate_min ?minutes ?steps WHERE { ?datapoint a health:Activity ; health:activity_date ?date ; health:activity_id ?id . OPTIONAL { ?datapoint health:calories_burned ?calories_burned . } OPTIONAL { ?datapoint health:distance ?distance . } OPTIONAL { ?datapoint health:hearth_rate_avg ?heart_rate_avg . } OPTIONAL { ?datapoint health:hearth_rate_min ?heart_rate_min . } OPTIONAL { ?datapoint health:hearth_rate_max ?heart_rate_max . } OPTIONAL { ?datapoint health:steps ?steps . } OPTIONAL { ?datapoint health:minutes ?minutes } } ORDER BY DESC(?date)").await?;

        while let Ok(true) = cursor.next_future().await {
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
                            Date::parse(cursor.string(i).unwrap().as_str())?
                                .and_time_utc(Time::new(0, 0, 0).unwrap())
                                .format_iso8601()
                                .unwrap()
                                .as_str(),
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
                .update_future(
                    resource
                        .print_sparql_update(Some(manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        connection
            .update_future(
                "DELETE WHERE { ?datapoint a health:Activity; health:activity_date ?date };",
            )
            .await?;

        self.emit_by_name::<()>("activities-updated", &[]);
        Ok(())
    }

    /// Migrate `Activity`s from date to dateTime. This will set all entries where a date is set to the date at 00:00:00 at the local datetime.
    ///
    /// # Returns
    /// An error if querying the DB goes wrong.
    pub async fn migrate_weight_date_datetime(&self) -> Result<()> {
        let imp = self.imp();
        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        let cursor =
        connection.query_future("SELECT ?date ?weight WHERE { ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . } ORDER BY ?date").await?;

        while let Ok(true) = cursor.next_future().await {
            let resource = tracker::Resource::new(None);
            resource.set_uri("rdf:type", "health:WeightMeasurement");
            resource.set_string(
                "health:weight_datetime",
                Date::parse(cursor.string(0).unwrap().as_str())?
                    .and_time_utc(Time::new(0, 0, 0).unwrap())
                    .format_iso8601()
                    .unwrap()
                    .as_str(),
            );
            resource.set_double("health:weight", cursor.double(1));

            connection
                .update_future(
                    resource
                        .print_sparql_update(Some(manager), None)
                        .unwrap()
                        .as_str(),
                )
                .await?;
        }

        connection
            .update_future(
                "DELETE WHERE { ?datapoint a health:WeightMeasurement; health:weight_date ?date };",
            )
            .await?;

        self.emit_by_name::<()>("weights-updated", &[]);
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
        let imp = self.imp();
        let connection = imp.connection.get().unwrap();
        connection
            .update_future("DELETE WHERE { ?datapoint a health:WeightMeasurement }")
            .await?;
        connection
            .update_future("DELETE WHERE { ?datapoint a health:Activity }")
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
        let imp = self.imp();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_datetime",
            &activity.date().format_iso8601().unwrap().as_str(),
        );
        resource.set_int64(
            "health:activity_id",
            activity.activity_type().to_u32().unwrap().into(),
        );

        if let Some(c) = activity.calories_burned() {
            resource.set_int64("health:calories_burned", c.into());
        }
        if let Some(d) = activity.distance() {
            resource.set_int64("health:distance", d.get::<meter>() as i64);
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
        if activity.duration().as_minutes() != 0 {
            resource.set_int64("health:minutes", activity.duration().as_minutes());
        }
        if let Some(s) = activity.steps() {
            resource.set_int64("health:steps", s.into());
        }

        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        connection
            .update_future(
                resource
                    .print_sparql_update(Some(manager), None)
                    .unwrap()
                    .as_str(),
            )
            .await?;

        self.emit_by_name::<()>("activities-updated", &[]);
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
        let imp = self.imp();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:WeightMeasurement");
        resource.set_string(
            "health:weight_datetime",
            &weight.date.format_iso8601().unwrap().as_str(),
        );
        resource.set_double("health:weight", weight.weight.get::<kilogram>().into());

        let connection = imp.connection.get().unwrap();
        let manager = imp.manager.get().unwrap();

        connection
            .update_future(
                resource
                    .print_sparql_update(Some(manager), None)
                    .unwrap()
                    .as_str(),
            )
            .await?;

        self.emit_by_name::<()>("weights-updated", &[]);
        Ok(())
    }

    /// Connect to the tracker DB. This has to be called before calling any other methods on this struct.
    ///
    /// # Arguments
    /// * `ontology_path` - `Some` if a custom path for the Tracker ontology path is desired (e.g. in tests), or `None` to use the default.
    /// * `store_path` - `Some` if a custom store path for the Tracker DB is desired (e.g. in tests), or `None` to use the default.
    ///
    /// # Panics
    /// This function will panic if it's called on the same [Database] object multiple times.
    fn connect(&self, ontology_path: Option<PathBuf>, store_path: Option<PathBuf>) -> Result<()> {
        let imp = self.imp();
        let mut store_path = store_path.unwrap_or_else(glib::user_data_dir);
        store_path.push("health");

        let ontology_path = if let Ok(p) = std::env::var("HEALTH_ONTOLOGY_PATH") {
            Path::new(&p).to_path_buf()
        } else {
            let mut ontology_path =
                ontology_path.unwrap_or_else(|| Path::new(crate::config::PKGDATADIR).to_path_buf());
            ontology_path.push("ontology");
            ontology_path
        };

        let manager = tracker::NamespaceManager::new();
        manager.add_prefix("health", "https://gitlab.gnome.org/World/health#");

        imp.manager.set(manager).unwrap();
        imp.connection
            .set(tracker::SparqlConnection::new(
                tracker::SparqlConnectionFlags::NONE,
                Some(&gio::File::for_path(store_path)),
                Some(&gio::File::for_path(ontology_path)),
                None::<&gio::Cancellable>,
            )?)
            .unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{cell::Cell, rc::Rc};

    use super::*;
    use crate::model::ActivityType;
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
        let data_dir = tempdir().unwrap();
        let date = glib::DateTime::local();
        let expected_activity = Activity::builder()
            .activity_type(ActivityType::Walking)
            .date(date.clone())
            .build();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

        let retrieved_activities = async move {
            db.save_activity(expected_activity).await.unwrap();

            db.activities(Some(date.add_days(1).unwrap()))
                .await
                .unwrap()
        }
        .block();
        assert!(retrieved_activities.is_empty());
    }

    #[test]
    fn check_doesnt_exists_weight() {
        let data_dir = tempdir().unwrap();
        let date = glib::DateTime::local();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.clone(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = async move {
            db.save_weight(w).await.unwrap();

            db.weights(Some(date.add_days(1).unwrap())).await.unwrap()
        }
        .block();
        assert!(retrieved_weights.is_empty());
    }

    #[test]
    fn check_exists_activity() {
        let data_dir = tempdir().unwrap();
        let date = glib::DateTime::local();
        let expected_activity = Activity::builder()
            .activity_type(ActivityType::Walking)
            .date(date.clone())
            .steps(50)
            .build();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();

        let a = expected_activity.clone();

        let retrieved_activities = async move {
            db.save_activity(a).await.unwrap();

            db.activities(Some(date.add_days(-1).unwrap()))
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
        let date = glib::DateTime::local();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let expected_weight = Weight::new(date.clone(), Mass::new::<kilogram>(50.0));
        let w = expected_weight.clone();

        let retrieved_weights = async move {
            db.save_weight(w).await.unwrap();

            db.weights(Some(date.add_days(-1).unwrap())).await.unwrap()
        }
        .block();
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
    }

    #[test]
    fn migration_activities() {
        let data_dir = tempdir().unwrap();
        let date = glib::DateTime::local();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.connection();
        let expected_activity = Activity::builder()
            .activity_type(ActivityType::Walking)
            .date(date.clone())
            .steps(50)
            .build();
        let manager = db.manager();
        let resource = tracker::Resource::new(None);

        resource.set_uri("rdf:type", "health:Activity");
        resource.set_string(
            "health:activity_date",
            &expected_activity.date().format("%Y-%m-%d").unwrap(),
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
            db.activities(Some(date.add_days(-1).unwrap()))
                .await
                .unwrap()
        }
        .block();
        let activity = retrieved_activities.get(0).unwrap();
        assert_eq!(expected_activity.steps(), activity.steps());
        assert_eq!(
            expected_activity
                .date()
                .reset_hms()
                .format_iso8601()
                .unwrap(),
            activity.date().reset_hms().format_iso8601().unwrap()
        );
        assert_eq!(
            expected_activity.activity_type().to_u32().unwrap(),
            activity.activity_type().to_u32().unwrap()
        );
    }

    #[test]
    fn migration_weights() {
        let data_dir = tempdir().unwrap();
        let date = glib::DateTime::local();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let connection = db.connection();
        let expected_weight = Weight::new(date.clone(), Mass::new::<kilogram>(50.0));
        let manager = db.manager();
        let resource = tracker::Resource::new(None);
        resource.set_uri("rdf:type", "health:WeightMeasurement");
        resource.set_string(
            "health:weight_date",
            &expected_weight.date.format("%Y-%m-%d").unwrap(),
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
            db.weights(Some(date.add_days(-1).unwrap())).await.unwrap()
        }
        .block();
        let weight = retrieved_weights.get(0).unwrap();
        assert_eq!(expected_weight.weight, weight.weight);
        assert_eq!(
            expected_weight.date.reset_hms().format_iso8601().unwrap(),
            weight.date.reset_hms().format_iso8601().unwrap()
        );
    }

    #[test]
    fn test_connect_activities_updated() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let was_called = Rc::new(Cell::new(false));
        let activity = Activity::new();

        db.connect_activities_updated(glib::clone!(@weak was_called => move |_| {
            was_called.set(true);
        }));
        async move {
            db.save_activity(activity).await.unwrap();
        }
        .block();
        assert!(was_called.get());
    }

    #[test]
    fn test_connect_weights_updated() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let was_called = Rc::new(Cell::new(false));
        let date = glib::DateTime::local();
        let weight = Weight::new(date.into(), Mass::new::<kilogram>(50.0));

        db.connect_weights_updated(glib::clone!(@weak was_called => move |_| {
            was_called.set(true);
        }));
        async move {
            db.save_weight(weight).await.unwrap();
        }
        .block();
        assert!(was_called.get());
    }

    #[test]
    #[allow(unused_braces)]
    fn test_calories() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let activity = Activity::new();
        let date = activity.date().add_days(-1).unwrap();

        activity.set_calories_burned(Some(500));
        glib::clone!(@weak db, @weak activity => async move {
                db.save_activity(activity).await.unwrap();
            }
        )
        .block();
        let calories = glib::clone!(@strong db, @strong date => async move { db.calories(date).await.unwrap() }).block();
        assert_eq!(*calories[0].calorie_split.values().next().unwrap(), 500);

        glib::clone!(@weak db, @weak activity => async move {
                db.save_activity(activity).await.unwrap();
            }
        )
        .block();
        let calories = glib::clone!(@strong db, @strong date => async move { db.calories(date).await.unwrap() }).block();
        assert_eq!(*calories[0].calorie_split.values().next().unwrap(), 1000);

        activity.set_date(date.clone());
        glib::clone!(@weak db, @weak activity => async move {
                db.save_activity(activity).await.unwrap();
            }
        )
        .block();
        let cloned = db.clone();
        let calories = async move { cloned.calories(date).await.unwrap() }.block();
        assert_eq!(calories.len(), 2);
        assert_eq!(*calories[0].calorie_split.values().next().unwrap(), 500);
        assert_eq!(*calories[1].calorie_split.values().next().unwrap(), 1000);
    }

    #[test]
    fn test_most_frequent_activities() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let now = glib::DateTime::local();
        let activities = vec![
            (
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .calories_burned(5)
                    .date(now.clone())
                    .build(),
                vec![ActivityType::Walking],
            ),
            (
                Activity::builder()
                    .activity_type(ActivityType::Basketball)
                    .calories_burned(5)
                    .date(now.clone())
                    .build(),
                vec![ActivityType::Walking, ActivityType::Basketball],
            ),
            (
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .calories_burned(5)
                    .date(now.clone())
                    .build(),
                vec![ActivityType::Walking, ActivityType::Basketball],
            ),
            (
                Activity::builder()
                    .activity_type(ActivityType::Swimming)
                    .calories_burned(5)
                    .date(now.clone())
                    .build(),
                vec![
                    ActivityType::Walking,
                    ActivityType::Swimming,
                    ActivityType::Basketball,
                ],
            ),
        ];

        let prev = now.clone().add_minutes(-1).unwrap();

        for (activity, expected_types) in activities {
            glib::clone!(@weak db, @strong prev => async move {
                db.save_activity(activity).await.unwrap();
                assert_eq!(expected_types, db.most_frequent_activities(prev).await.unwrap());
            })
            .block();
        }
    }

    #[test]
    fn test_has_activities() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        glib::clone!(@weak db => async move {
            assert!(!db.has_activities().await.unwrap());
            db.save_activity(Activity::new()).await.unwrap();
            assert!(db.has_activities().await.unwrap());
        })
        .block();
    }

    #[test]
    fn test_todays_steps() {
        let data_dir = tempdir().unwrap();
        let database = Database::new_with_store_path(data_dir.path().into()).unwrap();
        let db = database.clone();
        async move {
            let now = glib::DateTime::local();
            assert_eq!(db.todays_steps().await.unwrap(), 0);
            db.save_activity(
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .steps(1000)
                    .date(now.clone())
                    .build(),
            )
            .await
            .unwrap();
            assert_eq!(db.todays_steps().await.unwrap(), 1000);
            db.save_activity(
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .steps(1000)
                    .date(now.clone())
                    .build(),
            )
            .await
            .unwrap();
            assert_eq!(db.todays_steps().await.unwrap(), 2000);
            db.save_activity(
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .steps(1500)
                    .date(now.clone())
                    .build(),
            )
            .await
            .unwrap();
            assert_eq!(db.todays_steps().await.unwrap(), 3500);
            db.save_activity(
                Activity::builder()
                    .activity_type(ActivityType::Walking)
                    .steps(1500)
                    .date(now.add_days(-1).unwrap())
                    .build(),
            )
            .await
            .unwrap();
            assert_eq!(db.todays_steps().await.unwrap(), 3500);
        }
        .block();
    }

    #[test]
    fn test_weight_exists_on_date() {
        let data_dir = tempdir().unwrap();
        let db = Database::new_with_store_path(data_dir.path().into()).unwrap();
        glib::clone!(@weak db => async move {
            let now = glib::DateTime::local();
            let mass = Mass::new::<kilogram>(70.0);
            db.save_weight(Weight::new(now.clone().add_days(-1).unwrap(), mass)).await.unwrap();
            assert!(!db.weight_exists_on_date(now.clone()).await.unwrap());
            db.save_weight(Weight::new(now.clone(), mass)).await.unwrap();
            assert!(db.weight_exists_on_date(now).await.unwrap());
        })
        .block();
    }
}
