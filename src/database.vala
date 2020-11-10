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
        OPEN_FAILED,
        SETUP_FAILED,
        SAVE_FAILED,
        GET_FAILED,
        DATA_MALFORMED,
        IMPORT_FAILED,
    }

    /**
     * Gets the default location of the DB.
     */
    public string get_default_path () {
        GLib.DirUtils.create_with_parents (
            GLib.Path.build_filename (GLib.Environment.get_user_data_dir (), "gnome-health"),
            0755
        );
        return GLib.Path.build_filename (GLib.Environment.get_user_data_dir (), "gnome-health", "health_data.db");
    }

    public interface Database {
        /**
         * Gets all step records that have a date >= the date parameter
         *
         * If date is 30 of September 2020 then all step records that have been
         * added on the 30th of September or later will be returned.
         *
         * @param date The earliest date that steps should be retrieved from.
         * @throws DatabaseError If querying the DB fails.
         */
        public abstract Gee.ArrayList<Steps> get_steps_after (GLib.Date date) throws DatabaseError;

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
        public abstract Gee.ArrayList<Weight> get_weights_after (GLib.Date date, Settings settings) throws DatabaseError;

        /**
         * Opens the database located at filename, or creates a new one.
         *
         *
         * @param filename The filename (including path) to the db, e.g. `/tmp/gnome_health.db`
         * @throws DatabaseError If opening or creating the DB fails.
         */
        public abstract void open (string filename = get_default_path ()) throws DatabaseError;

        /**
         * Saves a `Weight` to the DB. Updates the weight if there's already one for the weight's date.
         *
         * @param w The `Weight` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public abstract void save_weight (Weight w) throws DatabaseError;

        /**
         * Saves a `Steps` to the DB. Updates the steps if there's already one for the steps's date.
         *
         * @param s The `Steps` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public abstract void save_steps (Steps s) throws DatabaseError;

        /**
         * Checks if there's already a step record for that date
         *
         * @param d The Date to check for
         * @return True if there's already a record, false otherwise.
         */
        public abstract bool check_steps_exist_on_date (Date d) throws DatabaseError;

        /**
         * Checks if there's already a weight measurement for that date
         *
         * @param d The Date to check for
         * @return True if there's already a measurement, false otherwise.
         */
        public abstract bool check_weight_exist_on_date (Date d) throws DatabaseError;

        public abstract void import_steps (Gee.ArrayList<Steps> s) throws DatabaseError;

        public abstract void import_weights (Gee.ArrayList<Weight> w) throws DatabaseError;
    }

    public class TrackerDatabase : GLib.Object {

        private TrackerDatabase () throws DatabaseError {
            var ontology_path = GLib.Path.build_filename (Config.DATADIR, "ontology");
            var store_path = GLib.Path.build_filename (GLib.Path.build_filename (GLib.Environment.get_user_data_dir (), "health"));

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

        public static TrackerDatabase get_instance () throws DatabaseError {
            if (instance == null) {
                instance = new TrackerDatabase ();
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
            resource.set_uri("rdf:type", "health:DataPoint");
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

    /**
     * An Implementor of the {@link Database} interface that uses SQLite as backend.
     */
    public class SqliteDatabase : GLib.Object, Database {
        /**
         * {@inheritDoc}
         */
        public void open (string filename = get_default_path ()) throws DatabaseError {
            int rc;
            const string SETUP_QUERY = """
                CREATE TABLE HealthData (
                    date            PRIMARY_KEY     INT     NOT NULL UNIQUE,
                    steps                           INT,
                    weight                          REAL
                );
            """;
            string? errmsg;
            bool db_exists_already = FileUtils.test (filename, FileTest.IS_REGULAR);

            if ((rc = Sqlite.Database.open_v2 (filename, out this.db)) != 0) {
                throw new DatabaseError.OPEN_FAILED (_ ("Opening the SQLite database failed to error %s"), this.db.errmsg ());
            }

            if (!db_exists_already) {
                if ((rc = this.db.exec (SETUP_QUERY, null, out errmsg)) != Sqlite.OK) {
                    throw new DatabaseError.SETUP_FAILED (_ ("Failed to setup SQLite database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
                }
            }

        }

        /**
         * {@inheritDoc}
         */
        public void save_weight (Weight w) throws DatabaseError {
            string query = "INSERT INTO HealthData (date, weight) VALUES (%u, %lf) ON CONFLICT(date) DO UPDATE SET weight=excluded.weight;".printf (w.date.get_julian (), w.weight.get_in_kg ());
            int rc;
            string? errmsg;

            if ((rc = this.db.exec (query, null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.SAVE_FAILED (_ ("Failed to save weight to SQLite database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }
        }

        /**
         * {@inheritDoc}
         */
        public void save_steps (Steps s) throws DatabaseError {
            string query = "INSERT INTO HealthData (date, steps) VALUES (%u, %u) ON CONFLICT(date) DO UPDATE SET steps=excluded.steps;".printf (s.date.get_julian (), s.steps);
            int rc;
            string? errmsg;

            if ((rc = this.db.exec (query, null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.SAVE_FAILED (_ ("Failed to save steps to SQLite database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }
        }

        /**
         * {@inheritDoc}
         */
        public Gee.ArrayList<Weight> get_weights_after (GLib.Date date, Settings settings) throws DatabaseError {
            int rc;
            Sqlite.Statement stmt;
            string query = "SELECT * FROM HealthData WHERE date >= %u AND weight IS NOT NULL;".printf (date.get_julian ());

            if ((rc = this.db.prepare_v2 (query, -1, out stmt, null)) == 1) {
                throw new DatabaseError.GET_FAILED (_ ("Failed to get weights from SQLite database due to error %s"), this.db.errmsg ());
            }

            var ret = new Gee.ArrayList<Weight> ();
            do {
                rc = stmt.step ();
                switch (rc) {
                case Sqlite.DONE:
                    break;
                case Sqlite.ROW:
                    var new_date = GLib.Date ();
                    new_date.set_julian (stmt.column_int (0));
                    ret.add (new Weight (new_date, new WeightUnitContainer.from_database_value ( stmt.column_double (2), settings)));
                    break;
                default:
                    throw new DatabaseError.GET_FAILED (_ ("Failed to get data from column due to error %s"), this.db.errmsg ());
                }
            } while (rc == Sqlite.ROW);
            return ret;
        }

        /**
         * {@inheritDoc}
         */
        public Gee.ArrayList<Steps> get_steps_after (GLib.Date date) throws DatabaseError {
            int rc;
            Sqlite.Statement stmt;
            string query = "SELECT * FROM HealthData WHERE date >= %u AND steps IS NOT NULL;".printf (date.get_julian ());

            if ((rc = this.db.prepare_v2 (query, -1, out stmt, null)) == 1) {
                throw new DatabaseError.GET_FAILED (_ ("Failed to get steps from SQLite database due to error %s"), this.db.errmsg ());
            }

            var ret = new Gee.ArrayList<Steps> ();
            do {
                rc = stmt.step ();
                switch (rc) {
                case Sqlite.DONE:
                    break;
                case Sqlite.ROW:
                    var new_date = GLib.Date ();
                    new_date.set_julian (stmt.column_int (0));
                    ret.add (new Steps (new_date, (uint32) stmt.column_int64 (1)));
                    break;
                default:
                    throw new DatabaseError.GET_FAILED (_ ("Failed to get data from column due to error %s"), this.db.errmsg ());
                }
            } while (rc == Sqlite.ROW);
            return ret;
        }

        /**
         * {@inheritDoc}
         */
        public bool check_steps_exist_on_date (GLib.Date d) throws DatabaseError {
            int rc;
            Sqlite.Statement stmt;
            string query = "SELECT EXISTS(SELECT 1 FROM HealthData WHERE date = %u AND steps IS NOT NULL);".printf (d.get_julian ());

            if ((rc = this.db.prepare_v2 (query, -1, out stmt, null)) == 1) {
                throw new DatabaseError.GET_FAILED (_ ("Failed to check if steps exist on date in database due to error %s"), this.db.errmsg ());
            }

            stmt.step ();
            return stmt.column_int (0) == 1;
        }

        /**
         * {@inheritDoc}
         */
        public bool check_weight_exist_on_date (GLib.Date d) throws DatabaseError {
            int rc;
            Sqlite.Statement stmt;
            string query = "SELECT EXISTS(SELECT 1 FROM HealthData WHERE date = %u AND weight IS NOT NULL);".printf (d.get_julian ());

            if ((rc = this.db.prepare_v2 (query, -1, out stmt, null)) == 1) {
                throw new DatabaseError.GET_FAILED (_ ("Failed to check if steps exist on date in database due to error %s"), this.db.errmsg ());
            }

            stmt.step ();
            return stmt.column_int (0) == 1;
        }

        public void import_steps (Gee.ArrayList<Steps> s) throws DatabaseError {
            int rc;
            string? errmsg = null;

            if ((rc = this.db.exec ("BEGIN TRANSACTION", null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import steps into database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }

            Sqlite.Statement stmt;
            const string QUERY = "INSERT INTO HealthData (date, steps) VALUES ($DATE, $STEPS) ON CONFLICT(date) DO UPDATE SET steps=excluded.steps;";
            if ((rc = this.db.prepare_v2 (QUERY, -1, out stmt, null)) == 1) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import steps into database due to error %s"), this.db.errmsg ());
            }

            var date_pos = stmt.bind_parameter_index ("$DATE");
            var steps_pos = stmt.bind_parameter_index ("$STEPS");
            assert (date_pos > 0);
            assert (steps_pos > 0);
            foreach (var step in s) {
                stmt.bind_int64 (date_pos, step.date.get_julian ());
                stmt.bind_int64 (steps_pos, step.steps);
                stmt.step ();
                stmt.clear_bindings ();
                stmt.reset ();
            }

            if ((rc = this.db.exec ("END TRANSACTION", null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import steps into database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }
        }

        public void import_weights (Gee.ArrayList<Weight> w) throws DatabaseError {
            int rc;
            string? errmsg = null;

            if ((rc = this.db.exec ("BEGIN TRANSACTION", null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import weights into database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }

            Sqlite.Statement stmt;
            const string QUERY = "INSERT INTO HealthData (date, weight) VALUES ($DATE, $WEIGHT) ON CONFLICT(date) DO UPDATE SET weight=excluded.weight;";
            if ((rc = this.db.prepare_v2 (QUERY, -1, out stmt, null)) == 1) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import weights into database due to error %s"), this.db.errmsg ());
            }

            var date_pos = stmt.bind_parameter_index ("$DATE");
            var weight_pos = stmt.bind_parameter_index ("$WEIGHT");
            assert (date_pos > 0);
            assert (weight_pos > 0);
            foreach (var weight in w) {
                stmt.bind_int64 (date_pos, weight.date.get_julian ());
                stmt.bind_double (weight_pos, weight.weight.get_in_kg ());
                stmt.step ();
                stmt.clear_bindings ();
                stmt.reset ();
            }

            if ((rc = this.db.exec ("END TRANSACTION", null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.IMPORT_FAILED (_ ("Failed to import weights into database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }
        }

        private Sqlite.Database db;
    }
}
