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
         * @throws DatabaseError If querying the DB fails.
         */
        public abstract Gee.ArrayList<Weight> get_weights_after (GLib.Date date) throws DatabaseError;

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

    }

    public class SqliteDatabase : GLib.Object, Database {
        /**
         * Opens the database located at filename, or creates a new one.
         *
         *
         * @param filename The filename (including path) to the db, e.g. `/tmp/gnome_health.db`
         * @throws DatabaseError If opening or creating the DB fails.
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
         * Saves a `Weight` to the DB. Updates the weight if there's already one for the weight's date.
         *
         * @param w The `Weight` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
         */
        public void save_weight (Weight w) throws DatabaseError {
            string query = "INSERT INTO HealthData (date, weight) VALUES (%u, %lf) ON CONFLICT(date) DO UPDATE SET weight=excluded.weight;".printf (w.date.get_julian (), w.weight);
            int rc;
            string? errmsg;

            if ((rc = this.db.exec (query, null, out errmsg)) != Sqlite.OK) {
                throw new DatabaseError.SAVE_FAILED (_ ("Failed to save weight to SQLite database due to error %s"), errmsg == null ? _ ("Unknown error") : (!) errmsg);
            }
        }

        /**
         * Saves a `Steps` to the DB. Updates the steps if there's already one for the steps's date.
         *
         * @param s The `Steps` that should be saved.
         * @throws DatabaseError If saving to the DB fails.
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
         * Gets all weight records that have a date >= the date parameter
         *
         * If date is 30 of September 2020 then all weight records that have been
         * added on the 30th of September or later will be returned.
         *
         * @param date The earliest date that steps should be retrieved from.
         * @throws DatabaseError If querying the DB fails.
         */
        public Gee.ArrayList<Weight> get_weights_after (GLib.Date date) throws DatabaseError {
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
                    ret.add (new Weight (new_date, stmt.column_double (2)));
                    break;
                default:
                    throw new DatabaseError.GET_FAILED (_ ("Failed to get data from column due to error %s"), this.db.errmsg ());
                }
            } while (rc == Sqlite.ROW);
            return ret;
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

        private Sqlite.Database db;
    }
}
