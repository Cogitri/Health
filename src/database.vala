/* database.vala
 *
 * Copyright 2020 Rasmus Thomsen <oss@cogitri.dev>
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

namespace Health {

    public errordomain DatabaseError {
        SETUP_FAILED,
    }

    public class TrackerDatabase : GLib.Object {

        private TrackerDatabase (string store_path = GLib.Path.build_filename (GLib.Environment.get_user_data_dir (), "health")) throws DatabaseError {
            string? ontology_path = "";

            if ((ontology_path = GLib.Environ.get_variable (GLib.Environ.get (), "HEALTH_ONTOLOGY_OVERRIDE_PATH")) == null) {
                ontology_path = GLib.Path.build_filename (Config.DATADIR, "ontology");
            }

            try {
                this.db = Tracker.Sparql.Connection.new (0, GLib.File.new_for_path (store_path), GLib.File.new_for_path ((!) ontology_path), null);
            } catch (GLib.Error e) {
                throw new DatabaseError.SETUP_FAILED (_ ("Failed to setup Tracker database due to error %s!").printf (e.message));
            }


            this.manager = new Tracker.NamespaceManager ();
            this.manager.add_prefix ("health", "https://gitlab.gnome.org/World/health#");
        }

        /**
         * Checks if there's already a step record for that date
         *
         * @param d The Date to check for
         * @return True if there's already a record, false otherwise.
         */
        public async bool check_steps_exist_on_date (Date d, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_DATE_HAS_STEPS.printf (date_to_iso_8601 (d)), cancellable);

            assert (yield cursor.next_async (cancellable));

            return cursor.get_boolean (0);
        }

        /**
         * Checks if there's already a weight measurement for that date
         *
         * @param d The Date to check for
         * @return True if there's already a measurement, false otherwise.
         */
        public async bool check_weight_exist_on_date (Date d, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_DATE_HAS_WEIGHT.printf (date_to_iso_8601 (d)), cancellable);

            assert (yield cursor.next_async (cancellable));

            return cursor.get_boolean (0);
        }

        public static TrackerDatabase get_instance (string store_path = GLib.Path.build_filename (GLib.Environment.get_user_data_dir (), "health")) throws DatabaseError {
            if (instance == null) {
                instance = new TrackerDatabase (store_path);
            }

            return instance;
        }

        /**
         * Gets all step records that have a date >= the date parameter
         *
         * If date is 30 of September 2020 then all step records that have been
         * added on the 30th of September or later will be returned.
         *
         * @param date The earliest date that steps should be retrieved from.
         * @throws DatabaseError If querying the DB fails.
         */
        public async Gee.ArrayList<Steps> get_steps_after (GLib.Date date, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_STEPS_AFTER.printf (date_to_iso_8601 (date)), cancellable);
            var hashmap = new Gee.HashMap<string, uint32> ();
            var ret = new Gee.ArrayList<Steps> ();

            while (yield cursor.next_async (cancellable)) {
                var date_string = cursor.get_string (0);
                if (hashmap.has_key (date_string)) {
                    hashmap.set (date_string, hashmap.get (date_string) + (uint32) cursor.get_integer (1));
                } else {
                    hashmap.set (date_string, (uint32) cursor.get_integer (1));
                }
            }

            foreach (var kv in hashmap) {
                ret.add (new Steps (iso_8601_to_date (kv.key), kv.value));
            }

            ret.sort ((a, b) => { return a.date.compare (b.date); });

            return ret;
        }

        public async uint32? get_steps_on_date (GLib.Date d, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_STEPS_ON_DAY.printf (date_to_iso_8601 (d)), cancellable);

            uint32? steps = null;

            while (yield cursor.next_async (cancellable)) {
                steps = steps ?? 0 + (uint32) cursor.get_integer (0);
            }

            return steps;
        }


        /**
         * Gets all weight records that have a date >= the date parameter
         *
         * If date is 30 of September 2020 then all weight records that have been
         * added on the 30th of September or later will be returned.
         *
         * @param date The earliest date that steps should be retrieved from.
         * @param settings The Health.Settings object that is used for determining whether to use imperial or metric units.
         * @throws DatabaseError If querying the DB fails.
         */
        public async Gee.ArrayList<Weight> get_weights_after (GLib.Date date, Settings settings, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_WEIGHT_AFTER.printf (date_to_iso_8601 (date)), cancellable);

            var ret = new Gee.ArrayList<Weight> ();
            while (yield cursor.next_async (cancellable)) {
                ret.add (new Weight (iso_8601_to_date (cursor.get_string (0)), new WeightUnitContainer.from_database_value ((uint32) cursor.get_double (1), settings)));
            }

            return ret;
        }

        public async double? get_weight_on_date (GLib.Date d, GLib.Cancellable? cancellable) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_WEIGHT_ON_DAY.printf (date_to_iso_8601 (d)), cancellable);

            if (yield cursor.next_async (cancellable)) {
                return cursor.get_double (0);
            } else {
                return null;
            }
        }

        public async void import_data (Gee.HashMap<string, uint32> steps, Gee.HashMap<string, double?> weight, GLib.Cancellable? cancellable = null) throws GLib.Error {
            string[] ops = {};

            info ("Importing %u step counts and %u weight measurements", steps.size, weight.size);


            foreach (var s in steps) {
                var resource = new Tracker.Resource (null);
                resource.set_uri ("rdf:type", "health:Activity");
                resource.set_string ("health:activity_date", s.key);
                resource.set_int64 ("health:steps", s.value);
                resource.set_int64 ("health:activity_id", Activities.Enum.WALKING);
                // FIXME: Set correct minutes here
                resource.set_int64 ("health:minutes", 0);

                ops += resource.print_sparql_update (this.manager, null);
            }

            // ...and afterwards add all the weight records which don't have a step record on that date.
            foreach (var w in weight) {
                var resource = new Tracker.Resource (null);
                resource.set_uri ("rdf:type", "health:WeightMeasurement");
                resource.set_string ("health:weight_date", w.key);
                resource.set_double ("health:weight", w.value);

                ops += resource.print_sparql_update (this.manager, null);
            }

            yield this.db.update_array_async (ops, cancellable);
            this.steps_updated ();
            this.weight_updated ();
        }

        public async void reset () throws GLib.Error {
            yield this.db.update_async ("DELETE WHERE { ?datapoint a health:WeightMeasurement }");
            yield this.db.update_async ("DELETE WHERE { ?datapoint a health:Activity }");
        }

        /**
         * Saves an `Activity` to the DB. Updates the steps if there's already one for the steps's date.
         *
         * @param s The `Steps` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
         public async void save_activity (Activity a, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var resource = new Tracker.Resource (null);
            resource.set_uri ("rdf:type", "health:Activity");
            resource.set_string ("health:activity_date", date_to_iso_8601 (a.date));
            if (a.steps != 0) {
                resource.set_int64 ("health:steps", a.steps);
            }
            resource.set_int64 ("health:activity_id", a.activity_type);
            resource.set_int64 ("health:minutes", a.minutes);

            yield this.db.update_async (resource.print_sparql_update (this.manager, null));
            this.steps_updated ();
        }

        /**
         * Saves a `Weight` to the DB. Updates the weight if there's already one for the weight's date.
         *
         * @param w The `Weight` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public async void save_weight (Weight w, GLib.Cancellable? cancellable = null) throws GLib.Error {
            var resource = new Tracker.Resource (null);
            resource.set_uri ("rdf:type", "health:WeightMeasurement");
            resource.set_string ("health:weight_date", date_to_iso_8601 (w.date));
            resource.set_double ("health:weight", w.weight.get_in_kg ());

            yield this.db.update_async ("DELETE WHERE { ?u health:weight_date '%s' }; %s".printf (date_to_iso_8601 (w.date), resource.print_sparql_update (this.manager, null)));
            this.weight_updated ();
        }

        public signal void steps_updated ();
        public signal void weight_updated ();

        const string QUERY_DATE_HAS_STEPS = "ASK { ?activity a health:Activity ; health:activity_date '%s'; health:steps ?steps . }";
        const string QUERY_DATE_HAS_WEIGHT = "ASK { ?datapoint a health:WeightMeasurement ; health:weight_date '%s'; health:weight ?weight . }";
        const string QUERY_STEPS_AFTER = "SELECT ?date ?steps WHERE { ?datapoint a health:Activity ; health:activity_date ?date ; health:steps ?steps . FILTER  (?date >= '%s'^^xsd:date)} ORDER BY ?date";
        const string QUERY_STEPS_ON_DAY = "SELECT ?steps WHERE { ?datapoint a health:Activity; health:activity_date ?date ; health:steps ?steps . FILTER(?date = '%s'^^xsd:date) }";
        const string QUERY_WEIGHT_AFTER = "SELECT ?date ?weight WHERE { ?datapoint a health:WeightMeasurement ; health:weight_date ?date  ; health:weight ?weight . FILTER  (?date >= '%s'^^xsd:date)} ORDER BY ?date";
        const string QUERY_WEIGHT_ON_DAY = "SELECT ?weight WHERE { ?datapoint a health:WeightMeasurement; health:weight_date ?date ; health:weight ?weight . FILTER(?date = '%s'^^xsd:date) }";


        private static TrackerDatabase instance;
        private Tracker.NamespaceManager manager;
        private Tracker.Sparql.Connection db;
    }

}
