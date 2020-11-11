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
    class TrackerDatabaseTest : ValaUnit.TestCase {
        public TrackerDatabaseTest () {
            base ("TrackerDatabaseTest");
            this.add_test ("check_exists_steps", this.check_exists_steps);
            this.add_test ("check_exists_weight", this.check_exists_weight);
            this.add_test ("get_after_steps", this.get_after_steps);
            this.add_test ("get_after_weight", this.get_after_weight);
            this.add_test ("save_steps", this.save_steps);
            this.add_test ("save_weight", this.save_weight);
        }

        public void check_exists_steps () throws ValaUnit.TestError {
            var db = this.open_db ();
            var date = get_today_date ();
            var steps = new Steps (date, 10000);
            try {
                db.save_steps.begin (steps, null, this.async_completion);
                db.save_steps.end (this.async_result ());
                db.check_steps_exist_on_date.begin (date, null, this.async_completion);
                assert (db.check_steps_exist_on_date.end (this.async_result ()));
                date.subtract_days (1);
                db.check_steps_exist_on_date.begin (date, null, this.async_completion);
                assert (!db.check_steps_exist_on_date.end (this.async_result ()));
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
        }

        public void check_exists_weight () throws ValaUnit.TestError {
            var db = this.open_db ();
            var date = get_today_date ();
            var settings = new Settings ();
            var weight = new Weight (date, new WeightUnitContainer.from_database_value (100, settings));
            try {
                db.save_weight.begin (weight, null, this.async_completion);
                db.save_weight.end (this.async_result ());
                db.check_weight_exist_on_date.begin (date, null, this.async_completion);
                assert (db.check_weight_exist_on_date.end (this.async_result ()));
                date.subtract_days (1);
                db.check_weight_exist_on_date.begin (date, null, this.async_completion);
                assert (!db.check_weight_exist_on_date.end (this.async_result ()));
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
        }

        public void get_after_steps () throws ValaUnit.TestError {
            var db = this.open_db ();
            var date = get_today_date ();
            var steps = new Steps (date, 10000);
            try {
                db.save_steps.begin (steps, null, this.async_completion);
                db.save_steps.end (this.async_result ());
                db.get_steps_after.begin (date, null, this.async_completion);
                assert_equal<uint?> (db.get_steps_after.end (this.async_result ()).size, 1);
                date.subtract_days (1);
                db.get_steps_after.begin (date, null, this.async_completion);
                assert (!db.get_steps_after.end (this.async_result ()).is_empty);
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
        }

        public void get_after_weight () throws ValaUnit.TestError {
            var db = this.open_db ();
            var date = get_today_date ();
            var settings = new Settings ();
            var weight = new Weight (date, new WeightUnitContainer.from_database_value (100, settings));
            try {
                db.save_weight.begin (weight, null, this.async_completion);
                db.save_weight.end (this.async_result ());
                db.get_weights_after.begin (date, settings, null, this.async_completion);
                assert_equal<uint?> (db.get_weights_after.end (this.async_result ()).size, 1);
                date.subtract_days (1);
                db.get_weights_after.begin (date, settings, null, this.async_completion);
                assert (!db.get_weights_after.end (this.async_result ()).is_empty);
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
        }

        public void save_steps () throws ValaUnit.TestError {
            var db = this.open_db ();
            var steps = new Steps (get_today_date (), 10000);
            Gee.ArrayList<Steps>? retrieved_steps = null;
            try {
                db.save_steps.begin (steps, null, this.async_completion);
                db.save_steps.end (this.async_result ());
                db.get_steps_after.begin (get_today_date (), null, this.async_completion);
                retrieved_steps = db.get_steps_after.end (this.async_result ());
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
            assert_equal<uint?> (((!) retrieved_steps).first ().date.get_julian (), steps.date.get_julian ());
            assert_equal<uint32?> (((!) retrieved_steps).first ().steps, steps.steps);
        }

        public void save_weight () throws ValaUnit.TestError {
            var db = this.open_db ();
            var settings = new Settings ();
            var weight = new Weight (get_today_date (), new WeightUnitContainer.from_database_value (100, settings));
            Gee.ArrayList<Weight>? retrieved_weight = null;
            try {
                db.save_weight.begin (weight, null, this.async_completion);
                db.save_weight.end (this.async_result ());
                db.get_weights_after.begin (get_today_date (), settings, null, this.async_completion);

                retrieved_weight = db.get_weights_after.end (this.async_result ());
            } catch (GLib.Error e) {
                try {
                    db.reset.begin (this.async_completion);
                    db.reset.end (this.async_result ());
                } catch (GLib.Error e) {
                    assert_no_error (e);
                }
                assert_no_error (e);
            }
            assert_equal<uint?> (((!) retrieved_weight).first ().date.get_julian (), weight.date.get_julian ());
            assert_equal<double?> (((!) retrieved_weight).first ().weight.value, weight.weight.value);
        }

        private TrackerDatabase open_db () throws ValaUnit.TestError {
            GLib.File? tmp_file = null;
            FileIOStream iostream;
            try {
                tmp_file = GLib.File.new_tmp (null, out iostream);
                // delete file so new db is created
                tmp_file.delete ();
            } catch (GLib.Error e) {
                assert_no_error (e);
            }
            try {
                var db = TrackerDatabase.get_instance ((!) tmp_file.get_path ());
                return db;
            } catch (GLib.Error e) {
                assert_no_error (e);
                return null;
            }
        }
    }
}
