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
            var ontology_path = GLib.Path.build_filename (Config.DATADIR, "ontology");

            try {
                this.db = Tracker.Sparql.Connection.new (0, GLib.File.new_for_path (store_path), GLib.File.new_for_path (ontology_path), null);
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
        public async bool check_steps_exist_on_date (Date d, GLib.Cancellable? cancellable) throws GLib.Error {
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
        public async bool check_weight_exist_on_date (Date d, GLib.Cancellable? cancellable) throws GLib.Error {
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
        public async Gee.ArrayList<Steps> get_steps_after (GLib.Date date, GLib.Cancellable? cancellable) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_STEPS_AFTER.printf (date_to_iso_8601 (date)), cancellable);

            var ret = new Gee.ArrayList<Steps> ();
            while (yield cursor.next_async (cancellable)) {
                ret.add (new Steps (iso_8601_to_date (cursor.get_string (0)), (uint32) cursor.get_integer (1)));
            }

            return ret;
        }

        public async uint32? get_steps_on_date (GLib.Date d, GLib.Cancellable? cancellable) throws GLib.Error {
            var cursor = yield this.db.query_async (QUERY_STEPS_ON_DAY.printf (date_to_iso_8601 (d)), cancellable);


            if (yield cursor.next_async (cancellable)) {
                return (uint32) cursor.get_integer (0);
            } else {
                return null;
            }
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
        public async Gee.ArrayList<Weight> get_weights_after (GLib.Date date, Settings settings, GLib.Cancellable? cancellable) throws GLib.Error {
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

        public async void import_steps (Gee.ArrayList<Steps> steps, GLib.Cancellable? cancellable) throws GLib.Error {
            foreach (var s in steps) {
                yield this.save_steps (s, cancellable);
            }
        }

        public async void import_weights (Gee.ArrayList<Weight> weight, GLib.Cancellable? cancellable) throws GLib.Error {
            foreach (var w in weight) {
                yield this.save_weight (w, cancellable);
            }
        }

        public async void reset () throws GLib.Error {
            yield this.db.update_async ("DELETE WHERE { ?datapoint a health:DataPoint }");
        }

        /**
         * Saves a `Weight` to the DB. Updates the weight if there's already one for the weight's date.
         *
         * @param w The `Weight` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public async void save_weight (Weight w, GLib.Cancellable? cancellable) throws GLib.Error {
            var resource = new Tracker.Resource (null);
            resource.set_uri ("rdf:type", "health:DataPoint");
            resource.set_string ("health:date", date_to_iso_8601 (w.date));
            resource.set_double ("health:weight", w.weight.get_in_kg ());

            var steps = yield this.get_steps_on_date (w.date, cancellable);
            if (steps != null) {
                resource.set_int64 ("health:steps", (!) steps);
            }

            yield this.db.update_async ("DELETE WHERE { ?u health:date '%s' }; %s".printf (date_to_iso_8601 (w.date), resource.print_sparql_update (this.manager, null)));
            this.weight_updated ();
        }

        /**
         * Saves a `Steps` to the DB. Updates the steps if there's already one for the steps's date.
         *
         * @param s The `Steps` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public async void save_steps (Steps s, GLib.Cancellable? cancellable) throws GLib.Error {
            var resource = new Tracker.Resource (null);
            resource.set_uri ("rdf:type", "health:DataPoint");
            resource.set_string ("health:date", date_to_iso_8601 (s.date));
            resource.set_int64 ("health:steps", s.steps);

            var weight = yield this.get_weight_on_date (s.date, cancellable);
            if (weight != null) {
                resource.set_double ("health:weight", (!) weight);
            }

            yield this.db.update_async ("DELETE WHERE { ?u health:date '%s' }; %s".printf (date_to_iso_8601 (s.date), resource.print_sparql_update (this.manager, null)));
            this.steps_updated ();
        }

        public signal void steps_updated ();
        public signal void weight_updated ();

        const string INSERT_STEPS = "INSERT OR REPLACE { ?datapoint a health:DataPoint ; health:date '%s' ; health:steps %s . } WHERE { SELECT ?datapoint WHERE { ?datapoint a health:DataPoint ; health:date ?date . FILTER ( ?date = '%s' ) } }";
        const string INSERT_WEIGHT = "INSERT OR REPLACE { ?datapoint a health:DataPoint ; health:date '%s' ; health:weight %s . } WHERE { SELECT ?datapoint WHERE { ?datapoint a health:DataPoint ; health:date ?date . FILTER ( ?date = '%s' ) } }";
        const string QUERY_DATE_HAS_STEPS = "ASK { ?datapoint a health:DataPoint ; health:date '%s'; health:steps ?steps . }";
        const string QUERY_DATE_HAS_WEIGHT = "ASK { ?datapoint a health:DataPoint ; health:date '%s'; health:weight ?weight . }";
        const string QUERY_STEPS_AFTER = "SELECT ?date ?steps WHERE { ?datapoint a health:DataPoint ; health:date ?date ; health:steps ?steps . FILTER  (?date >= '%s'^^xsd:date)} ORDER BY ?date";
        const string QUERY_STEPS_ON_DAY = "SELECT ?steps WHERE { ?datapoint a health:DataPoint; health:date ?date ; health:steps ?steps . FILTER(?date = '%s'^^xsd:date) }";
        const string QUERY_WEIGHT_AFTER = "SELECT ?date ?weight WHERE { ?datapoint a health:DataPoint ; health:date ?date  ; health:weight ?weight . FILTER  (?date >= '%s'^^xsd:date)} ORDER BY ?date";
        const string QUERY_WEIGHT_ON_DAY = "SELECT ?weight WHERE { ?datapoint a health:DataPoint; health:date ?date ; health:weight ?weight . FILTER(?date = '%s'^^xsd:date) }";

        private static TrackerDatabase instance;
        private Tracker.NamespaceManager manager;
        private Tracker.Sparql.Connection db;
    }

}
