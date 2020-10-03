/* add_dialog.vala
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

    /**
     * AddDialog is a generic dialog used for adding new data to the DB via user input.
     */
    public class AddDialog : Gtk.Dialog {
        protected Gtk.Label dialog_label;
        protected Gtk.Entry dialog_entry;

        public AddDialog (Gtk.Window? parent) {
            Object (use_header_bar: 1);
            this.set_transient_for (parent);
            this.destroy_with_parent = true;
            this.modal = true;
            this.dialog_label = new Gtk.Label (null);
            this.dialog_label.visible = true;
            this.dialog_entry = new Gtk.Entry ();
            this.dialog_entry.visible = true;
            this.dialog_entry.input_purpose = Gtk.InputPurpose.DIGITS;

            var content_box = this.get_content_area ();
            content_box.pack_start (this.dialog_label, true, true, 6);
            content_box.pack_start (this.dialog_entry, true, true, 6);

            this.add_button (_ ("Save"), Gtk.ResponseType.OK);
            this.add_button (_ ("Cancel"), Gtk.ResponseType.CANCEL);
            this.set_default_response (Gtk.ResponseType.OK);
            this.dialog_entry.changed.connect (() => {
                this.set_response_sensitive (Gtk.ResponseType.OK, this.dialog_entry.get_text_length () != 0);
            });

            this.response.connect ((response_id) => {
                switch (response_id) {
                case Gtk.ResponseType.OK:
                    try {
                        this.save ();
                    } catch (DatabaseError e) {
                        warning (_ ("Failed to save new data due to error %s"), e.message);
                    }
                    break;
                }
                this.destroy ();
            });
        }

        /**
         * save() should save the user's input to the DB.
         */
        public virtual void save () throws DatabaseError {
        }

    }

    /**
     * An `AddDialog` for adding a new step record.
     */
    public class StepsAddDialog : AddDialog {
        private SqliteDatabase db;

        public StepsAddDialog (Gtk.Window? parent, SqliteDatabase db) {
            base (parent);

            this.db = db;
            var update = false;
            try {
                update = db.check_steps_exist_on_date (get_today_date ());
            } catch (DatabaseError e) {
                warning (e.message);
            }

            if (update) {
                this.dialog_label.set_text (_ ("Update today's step record"));
            } else {
                this.dialog_label.set_text (_ ("Add new step record"));
            }
            this.dialog_entry.set_max_length (6);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public override void save () throws DatabaseError {
            var db = new SqliteDatabase ();
            db.open ();

            uint64 steps = 0;
            try {
                uint64.from_string (this.dialog_entry.get_text (), out steps);
            } catch (NumberParserError e) {
                warning (_("Failed to parse steps due to error %s"), e.message);
            }

            db.save_steps (new Steps (get_today_date (), (uint32) steps));
        }

    }

    /**
     * An `AddDialog` for adding a new weight record.
     */
    public class WeightAddDialog : AddDialog {
        private Settings settings;
        private SqliteDatabase db;

        public WeightAddDialog (Gtk.Window? parent, Settings settings, SqliteDatabase db) {
            base (parent);

            this.db = db;
            this.settings = settings;

            var update = false;
            try {
                update = db.check_weight_exist_on_date (get_today_date ());
            } catch (DatabaseError e) {
                warning (e.message);
            }

            if (update) {
                this.dialog_label.set_text (_ ("Update today's weight measurement"));
            } else {
                this.dialog_label.set_text (_ ("Add new weight measurement"));
            }

            this.dialog_entry.set_max_length (6);
        }

        /**
         * Saves the data that has been entered into the dialog to the database.
         */
        public override void save () throws DatabaseError {
            var db = new SqliteDatabase ();
            db.open ();

            double weight = 0;
            if (!double.try_parse (this.dialog_entry.get_text (), out weight)) {
                warning (_ ("Failed to parse weight '%s' as floating point number"), this.dialog_entry.get_text ());
            }

            db.save_weight (new Weight (get_today_date (), new WeightUnitContainer.from_user_value (weight, this.settings)));
        }

    }
}
